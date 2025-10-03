use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

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
    fn bergamot_service_pivot(
        service_ptr: *mut c_void,
        first_model_ptr: *mut c_void,
        second_model_ptr: *mut c_void,
        inputs: *const *const c_char,
        count: usize,
    ) -> *mut *mut c_char;
    fn bergamot_free_strings(strings: *mut *mut c_char, count: usize);
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

    pub fn translate(&self, model: &TranslationModel, inputs: &[String]) -> Vec<String> {
        let c_inputs: Vec<CString> = inputs
            .iter()
            .map(|s| CString::new(s.as_str()).expect("Failed to create CString"))
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

    pub fn pivot(
        &self,
        first_model: &TranslationModel,
        second_model: &TranslationModel,
        inputs: Vec<String>,
    ) -> Vec<String> {
        let c_inputs: Vec<CString> = inputs
            .iter()
            .map(|s| CString::new(s.as_str()).expect("Failed to create CString"))
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
