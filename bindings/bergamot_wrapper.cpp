#include <string>
#include <vector>
#include <memory>
#include "translator/parser.h"
#include "translator/response.h"
#include "translator/response_options.h"
#include "translator/service.h"

using namespace marian::bergamot;

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
}
