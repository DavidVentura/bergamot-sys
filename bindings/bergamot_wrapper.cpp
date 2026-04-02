#include <string>
#include <vector>
#include <memory>
#include <algorithm>
#include "translator/parser.h"
#include "translator/response.h"
#include "translator/response_options.h"
#include "translator/service.h"

using namespace marian::bergamot;

struct CTokenAlignment {
    size_t src_begin;
    size_t src_end;
    size_t tgt_begin;
    size_t tgt_end;
};

struct CTranslationWithAlignment {
    char* source;
    char* target;
    CTokenAlignment* alignments;
    size_t alignment_count;
};

extern "C" {
    void* bergamot_service_new(size_t cache_size) {
        BlockingService::Config config;
        config.cacheSize = cache_size;
        config.logger.level = "off";
        return new BlockingService(config);
    }

    void bergamot_service_delete(void* service_ptr) {
        delete static_cast<BlockingService*>(service_ptr);
    }

    void* bergamot_model_new(const char* config_yaml) {
        std::string cfg_str(config_yaml);
        auto validate = true;
        auto pathsDir = "";
        auto options = parseOptionsFromString(cfg_str, validate, pathsDir);
        return new TranslationModel(options);
    }

    void __attribute__ ((visibility ("default"))) bergamot_model_delete(void* model_ptr) {
        delete static_cast<TranslationModel*>(model_ptr);
    }

    char** bergamot_service_translate(void* service_ptr, void* model_ptr, const char** inputs, size_t count) {
        auto* service = static_cast<BlockingService*>(service_ptr);
        auto* model = static_cast<TranslationModel*>(model_ptr);

        std::vector<std::string> cpp_inputs;
        cpp_inputs.reserve(count);
        for (size_t i = 0; i < count; ++i) {
            cpp_inputs.emplace_back(inputs[i]);
        }

        std::vector<ResponseOptions> responseOptions;
        responseOptions.reserve(count);
        for (size_t i = 0; i < count; ++i) {
            ResponseOptions opts;
            opts.HTML = false;
            opts.qualityScores = false;
            opts.alignment = false;
            opts.sentenceMappings = false;
            responseOptions.emplace_back(opts);
        }

        std::shared_ptr<TranslationModel> model_shared(model, [](TranslationModel*){});
        std::vector<Response> responses = service->translateMultiple(model_shared, std::move(cpp_inputs), responseOptions);

        char** output = new char*[responses.size()];
        for (size_t i = 0; i < responses.size(); ++i) {
            const std::string& text = responses[i].target.text;
            output[i] = new char[text.length() + 1];
            strcpy(output[i], text.c_str());
        }

        return output;
    }

    char** bergamot_service_pivot(void* service_ptr, void* first_model_ptr, void* second_model_ptr, const char** inputs, size_t count) {
        auto* service = static_cast<BlockingService*>(service_ptr);
        auto* first_model = static_cast<TranslationModel*>(first_model_ptr);
        auto* second_model = static_cast<TranslationModel*>(second_model_ptr);

        std::vector<std::string> cpp_inputs;
        cpp_inputs.reserve(count);
        for (size_t i = 0; i < count; ++i) {
            cpp_inputs.emplace_back(inputs[i]);
        }

        std::vector<ResponseOptions> responseOptions;
        responseOptions.reserve(count);
        for (size_t i = 0; i < count; ++i) {
            ResponseOptions opts;
            opts.HTML = false;
            opts.qualityScores = false;
            opts.alignment = false;
            opts.sentenceMappings = false;
            responseOptions.emplace_back(opts);
        }

        std::shared_ptr<TranslationModel> first_shared(first_model, [](TranslationModel*){});
        std::shared_ptr<TranslationModel> second_shared(second_model, [](TranslationModel*){});
        std::vector<Response> responses = service->pivotMultiple(first_shared, second_shared, std::move(cpp_inputs), responseOptions);

        char** output = new char*[responses.size()];
        for (size_t i = 0; i < responses.size(); ++i) {
            const std::string& text = responses[i].target.text;
            output[i] = new char[text.length() + 1];
            strcpy(output[i], text.c_str());
        }

        return output;
    }

    void bergamot_free_strings(char** strings, size_t count) {
        for (size_t i = 0; i < count; ++i) {
            delete[] strings[i];
        }
        delete[] strings;
    }

    static void extractAlignments(const Response& resp, std::vector<CTokenAlignment>& out) {
        for (size_t s = 0; s < resp.source.numSentences(); ++s) {
            size_t numTarget = resp.target.numWords(s);
            size_t numSource = resp.source.numWords(s);
            if (numSource == 0) continue;

            for (size_t t = 0; t < numTarget; ++t) {
                ByteRange tgtRange = resp.target.wordAsByteRange(s, t);
                if (tgtRange.begin == tgtRange.end) continue; // skip EOS/empty tokens

                const auto& row = resp.alignments[s][t];
                size_t bestSrc = std::max_element(row.begin(), row.begin() + numSource) - row.begin();

                ByteRange srcRange = resp.source.wordAsByteRange(s, bestSrc);

                out.push_back(CTokenAlignment{srcRange.begin, srcRange.end, tgtRange.begin, tgtRange.end});
            }
        }
    }

    CTranslationWithAlignment* bergamot_service_translate_with_alignment(
            void* service_ptr, void* model_ptr, const char** inputs, size_t count) {
        auto* service = static_cast<BlockingService*>(service_ptr);
        auto* model = static_cast<TranslationModel*>(model_ptr);

        std::vector<std::string> cpp_inputs;
        cpp_inputs.reserve(count);
        for (size_t i = 0; i < count; ++i) {
            cpp_inputs.emplace_back(inputs[i]);
        }

        std::vector<ResponseOptions> responseOptions;
        responseOptions.reserve(count);
        for (size_t i = 0; i < count; ++i) {
            ResponseOptions opts;
            opts.HTML = false;
            opts.qualityScores = false;
            opts.alignment = true;
            opts.sentenceMappings = false;
            responseOptions.emplace_back(opts);
        }

        std::shared_ptr<TranslationModel> model_shared(model, [](TranslationModel*){});
        std::vector<Response> responses = service->translateMultiple(model_shared, std::move(cpp_inputs), responseOptions);

        CTranslationWithAlignment* results = new CTranslationWithAlignment[count];

        for (size_t i = 0; i < responses.size(); ++i) {
            const auto& resp = responses[i];

            results[i].source = new char[resp.source.text.length() + 1];
            strcpy(results[i].source, resp.source.text.c_str());
            results[i].target = new char[resp.target.text.length() + 1];
            strcpy(results[i].target, resp.target.text.c_str());

            std::vector<CTokenAlignment> alignments;
            extractAlignments(resp, alignments);

            results[i].alignment_count = alignments.size();
            results[i].alignments = new CTokenAlignment[alignments.size()];
            memcpy(results[i].alignments, alignments.data(), alignments.size() * sizeof(CTokenAlignment));
        }

        return results;
    }

    CTranslationWithAlignment* bergamot_service_pivot_with_alignment(
            void* service_ptr, void* first_model_ptr, void* second_model_ptr, const char** inputs, size_t count) {
        auto* service = static_cast<BlockingService*>(service_ptr);
        auto* first_model = static_cast<TranslationModel*>(first_model_ptr);
        auto* second_model = static_cast<TranslationModel*>(second_model_ptr);

        std::vector<std::string> cpp_inputs;
        cpp_inputs.reserve(count);
        for (size_t i = 0; i < count; ++i) {
            cpp_inputs.emplace_back(inputs[i]);
        }

        std::vector<ResponseOptions> responseOptions;
        responseOptions.reserve(count);
        for (size_t i = 0; i < count; ++i) {
            ResponseOptions opts;
            opts.HTML = false;
            opts.qualityScores = false;
            opts.alignment = true;
            opts.sentenceMappings = false;
            responseOptions.emplace_back(opts);
        }

        std::shared_ptr<TranslationModel> first_shared(first_model, [](TranslationModel*){});
        std::shared_ptr<TranslationModel> second_shared(second_model, [](TranslationModel*){});
        std::vector<Response> responses = service->pivotMultiple(first_shared, second_shared, std::move(cpp_inputs), responseOptions);

        CTranslationWithAlignment* results = new CTranslationWithAlignment[count];

        for (size_t i = 0; i < responses.size(); ++i) {
            const auto& resp = responses[i];

            results[i].source = new char[resp.source.text.length() + 1];
            strcpy(results[i].source, resp.source.text.c_str());
            results[i].target = new char[resp.target.text.length() + 1];
            strcpy(results[i].target, resp.target.text.c_str());

            std::vector<CTokenAlignment> alignments;
            extractAlignments(resp, alignments);

            results[i].alignment_count = alignments.size();
            results[i].alignments = new CTokenAlignment[alignments.size()];
            memcpy(results[i].alignments, alignments.data(), alignments.size() * sizeof(CTokenAlignment));
        }

        return results;
    }

    void bergamot_free_translations_with_alignment(CTranslationWithAlignment* results, size_t count) {
        for (size_t i = 0; i < count; ++i) {
            delete[] results[i].source;
            delete[] results[i].target;
            delete[] results[i].alignments;
        }
        delete[] results;
    }
}
