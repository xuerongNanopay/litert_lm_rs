use std::ffi::{CStr, CString};
use std::ptr;

fn main() {
    let model_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "models/gemma-4-E2B-it.litertlm".to_owned());

    let model_path = CString::new(model_path).expect("model path must not contain NUL bytes");
    let backend = CString::new("cpu").expect("backend must not contain NUL bytes");
    // Disable cache use: :nocache
    let cache_dir =
        CString::new("/tmp/litert-lm-cache/").expect("cache directory must not contain NUL bytes");
    let message = CString::new(
        r#"{"role":"user","content":"Write a Rust function that returns the nth Fibonacci number using an iterative approach. Include a short explanation."}"#,
    )
    .expect("message must not contain NUL bytes");
    let extra_context = CString::new("{}").expect("extra context must not contain NUL bytes");

    unsafe {
        let settings = litert_lm_sys::litert_lm_engine_settings_create(
            model_path.as_ptr(),
            backend.as_ptr(),
            ptr::null(),
            ptr::null(),
        );
        assert!(!settings.is_null(), "failed to create LiteRT-LM settings");

        litert_lm_sys::litert_lm_engine_settings_set_cache_dir(settings, cache_dir.as_ptr());

        litert_lm_sys::litert_lm_engine_settings_set_max_num_tokens(settings, 1024);

        let engine = litert_lm_sys::litert_lm_engine_create(settings);
        litert_lm_sys::litert_lm_engine_settings_delete(settings);
        assert!(!engine.is_null(), "failed to create LiteRT-LM engine");

        let conversation = litert_lm_sys::litert_lm_conversation_create(engine, ptr::null_mut());
        assert!(
            !conversation.is_null(),
            "failed to create LiteRT-LM conversation"
        );

        let response = litert_lm_sys::litert_lm_conversation_send_message(
            conversation,
            message.as_ptr(),
            extra_context.as_ptr(),
            ptr::null(),
        );
        assert!(!response.is_null(), "failed to generate a response");

        let response_json = litert_lm_sys::litert_lm_json_response_get_string(response);
        assert!(!response_json.is_null(), "response JSON is missing");

        println!("AAAAA: {}", CStr::from_ptr(response_json).to_string_lossy());

        litert_lm_sys::litert_lm_json_response_delete(response);
        litert_lm_sys::litert_lm_conversation_delete(conversation);
        litert_lm_sys::litert_lm_engine_delete(engine);
    }
}
