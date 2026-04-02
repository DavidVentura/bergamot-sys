use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TokenAlignment {
    pub src_begin: usize,
    pub src_end: usize,
    pub tgt_begin: usize,
    pub tgt_end: usize,
}

#[repr(C)]
struct CTranslationWithAlignment {
    source: *mut c_char,
    target: *mut c_char,
    alignments: *mut TokenAlignment,
    alignment_count: usize,
}

pub struct TranslationWithAlignment {
    pub source: String,
    pub target: String,
    pub alignments: Vec<TokenAlignment>,
}

unsafe extern "C" {
    fn bergamot_service_new(cache_size: usize) -> *mut c_void;
    fn bergamot_service_delete(service_ptr: *mut c_void);
    fn bergamot_model_new(config_yaml: *const c_char) -> *mut c_void;
    fn bergamot_model_delete(model_ptr: *mut c_void);
    fn bergamot_service_translate(
        service_ptr: *mut c_void,
        model_ptr: *mut c_void,
        inputs: *const *const c_char,
        count: usize,
    ) -> *mut *mut c_char;
    fn bergamot_service_translate_with_alignment(
        service_ptr: *mut c_void,
        model_ptr: *mut c_void,
        inputs: *const *const c_char,
        count: usize,
    ) -> *mut CTranslationWithAlignment;
    fn bergamot_service_pivot(
        service_ptr: *mut c_void,
        first_model_ptr: *mut c_void,
        second_model_ptr: *mut c_void,
        inputs: *const *const c_char,
        count: usize,
    ) -> *mut *mut c_char;
    fn bergamot_free_strings(strings: *mut *mut c_char, count: usize);
    fn bergamot_free_translations_with_alignment(
        results: *mut CTranslationWithAlignment,
        count: usize,
    );
}

/// Build a lookup table: byte_offset -> char_offset.
/// Index with any byte offset (0..=len) to get the corresponding char offset.
fn byte_to_char_offsets(s: &str) -> Vec<usize> {
    let mut table = vec![0usize; s.len() + 1];
    let mut char_idx = 0;
    for (byte_idx, _) in s.char_indices() {
        table[byte_idx] = char_idx;
        char_idx += 1;
    }
    table[s.len()] = char_idx; // for end-of-string
    // Fill gaps (mid-codepoint bytes) with the next char's index
    let mut last = char_idx;
    for i in (0..s.len()).rev() {
        if table[i] == 0 && i > 0 {
            table[i] = last;
        } else {
            last = table[i];
        }
    }
    table
}

pub struct TranslationModel {
    ptr: *mut c_void,
}

impl Drop for TranslationModel {
    fn drop(&mut self) {
        unsafe {
            bergamot_model_delete(self.ptr);
        }
    }
}

unsafe impl Send for TranslationModel {}
unsafe impl Sync for TranslationModel {}

impl TranslationModel {
    pub fn from_config(config: &str) -> Result<TranslationModel, String> {
        let c_config = CString::new(config).expect("Failed to create CString for config");
        let model_ptr = unsafe { bergamot_model_new(c_config.as_ptr()) };

        if model_ptr.is_null() {
            return Err("Failed to create translation model".to_string());
        }

        Ok(TranslationModel { ptr: model_ptr })
    }
}

pub struct BlockingService {
    ptr: *mut c_void,
}

impl BlockingService {
    pub fn new(cache_size: usize) -> Self {
        let ptr = unsafe { bergamot_service_new(cache_size) };
        assert!(!ptr.is_null(), "Failed to create blocking service");
        Self { ptr }
    }

    pub fn translate(&self, model: &TranslationModel, inputs: &[&str]) -> Vec<String> {
        let c_inputs: Vec<CString> = inputs
            .iter()
            .cloned()
            .map(|s| CString::new(s).expect("Failed to create CString"))
            .collect();

        let c_input_ptrs: Vec<*const c_char> = c_inputs.iter().map(|s| s.as_ptr()).collect();
        let count = c_input_ptrs.len();

        unsafe {
            let result_ptr =
                bergamot_service_translate(self.ptr, model.ptr, c_input_ptrs.as_ptr(), count);
            assert!(!result_ptr.is_null(), "Translation failed");

            let mut results = Vec::new();
            for i in 0..count {
                let c_str = CStr::from_ptr(*result_ptr.add(i));
                results.push(c_str.to_string_lossy().into_owned());
            }

            bergamot_free_strings(result_ptr, count);
            results
        }
    }

    pub fn translate_with_alignment(
        &self,
        model: &TranslationModel,
        inputs: &[&str],
    ) -> Vec<TranslationWithAlignment> {
        let c_inputs: Vec<CString> = inputs
            .iter()
            .cloned()
            .map(|s| CString::new(s).expect("Failed to create CString"))
            .collect();

        let c_input_ptrs: Vec<*const c_char> = c_inputs.iter().map(|s| s.as_ptr()).collect();
        let count = c_input_ptrs.len();

        unsafe {
            let result_ptr = bergamot_service_translate_with_alignment(
                self.ptr,
                model.ptr,
                c_input_ptrs.as_ptr(),
                count,
            );
            assert!(!result_ptr.is_null(), "Translation with alignment failed");

            let mut results = Vec::new();
            for i in 0..count {
                let c_result = &*result_ptr.add(i);

                let source = CStr::from_ptr(c_result.source)
                    .to_string_lossy()
                    .into_owned();
                let target = CStr::from_ptr(c_result.target)
                    .to_string_lossy()
                    .into_owned();
                let byte_alignments =
                    std::slice::from_raw_parts(c_result.alignments, c_result.alignment_count);

                let src_char_offsets = byte_to_char_offsets(&source);
                let tgt_char_offsets = byte_to_char_offsets(&target);

                let alignments = byte_alignments
                    .iter()
                    .map(|a| TokenAlignment {
                        src_begin: src_char_offsets[a.src_begin],
                        src_end: src_char_offsets[a.src_end],
                        tgt_begin: tgt_char_offsets[a.tgt_begin],
                        tgt_end: tgt_char_offsets[a.tgt_end],
                    })
                    .collect();

                results.push(TranslationWithAlignment {
                    source,
                    target,
                    alignments,
                });
            }

            bergamot_free_translations_with_alignment(result_ptr, count);
            results
        }
    }

    pub fn pivot(
        &self,
        first_model: &TranslationModel,
        second_model: &TranslationModel,
        inputs: &[&str],
    ) -> Vec<String> {
        let c_inputs: Vec<CString> = inputs
            .iter()
            .cloned()
            .map(|s| CString::new(s).expect("Failed to create CString"))
            .collect();

        let c_input_ptrs: Vec<*const c_char> = c_inputs.iter().map(|s| s.as_ptr()).collect();
        let count = c_input_ptrs.len();

        unsafe {
            let result_ptr = bergamot_service_pivot(
                self.ptr,
                first_model.ptr,
                second_model.ptr,
                c_input_ptrs.as_ptr(),
                count,
            );
            assert!(!result_ptr.is_null(), "Pivot translation failed");

            let mut results = Vec::new();
            for i in 0..count {
                let c_str = CStr::from_ptr(*result_ptr.add(i));
                results.push(c_str.to_string_lossy().into_owned());
            }

            bergamot_free_strings(result_ptr, count);
            results
        }
    }
}

impl Drop for BlockingService {
    fn drop(&mut self) {
        unsafe {
            bergamot_service_delete(self.ptr);
        }
    }
}

unsafe impl Send for BlockingService {}
unsafe impl Sync for BlockingService {}
