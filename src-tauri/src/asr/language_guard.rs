use super::{AsrError, TranscriptionRequest, TranscriptionResult};
use crate::settings::{LanguageMode, LanguageModeSetting};

/// Validate every mixed-language piece, including a forced retry. English stays
/// automatic because forcing all speech into the primary language can translate it.
pub async fn transcribe_in_chosen_languages<F, Fut>(
    language: &LanguageModeSetting,
    req: TranscriptionRequest,
    piece: &str,
    run: F,
) -> Result<TranscriptionResult, AsrError>
where
    F: Fn(TranscriptionRequest) -> Fut,
    Fut: std::future::Future<Output = Result<TranscriptionResult, AsrError>>,
{
    if language.mode != LanguageMode::Multi {
        return run(req).await;
    }
    let retry = TranscriptionRequest { language: language.primary().into(), ..req.clone() };
    let first = run(req).await?;
    if !language.needs_lock(first.detected_language.as_deref(), &first.text) {
        return Ok(first);
    }
    crate::journal::info("language.locked", serde_json::json!({
        "heard": first.detected_language, "forced": retry.language, "piece": piece,
    }));
    let again = run(retry).await.map_err(|e| {
        tracing::warn!("{piece}: language retry failed: {e}");
        AsrError::LanguageMismatch
    })?;
    if language.needs_lock(again.detected_language.as_deref(), &again.text) {
        crate::journal::warn("language.rejected", serde_json::json!({
            "heard": again.detected_language, "primary": language.primary(), "piece": piece,
        }));
        return Err(AsrError::LanguageMismatch);
    }
    Ok(TranscriptionResult {
        inference_ms: first.inference_ms.saturating_add(again.inference_ms),
        ..again
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn retry_preserves_audio_hints_and_adds_both_inference_times() {
        let language = LanguageModeSetting { mode: LanguageMode::Multi, primary: "el".into() };
        let request = TranscriptionRequest { wav: vec![1, 2, 3], language: "auto".into(), prompt: Some("Luram".into()), beam_size: 3, vad: false };
        let calls = AtomicUsize::new(0);
        let result = transcribe_in_chosen_languages(&language, request.clone(), "test", |req| {
            assert_eq!(req.wav, request.wav);
            assert_eq!(req.prompt, request.prompt);
            assert_eq!(req.beam_size, 3);
            assert!(!req.vad);
            let n = calls.fetch_add(1, Ordering::SeqCst);
            let (text, lang) = if n == 0 { assert_eq!(req.language, "auto"); ("Хара", "ru") }
                else { assert_eq!(req.language, "el"); ("Χαρά", "el") };
            std::future::ready(Ok(TranscriptionResult { text: text.into(), detected_language: Some(lang.into()), inference_ms: 20, ..Default::default() }))
        }).await.unwrap();
        assert_eq!(result.text, "Χαρά");
        assert_eq!(result.inference_ms, 40);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn unrestricted_auto_keeps_other_languages_and_uses_one_pass() {
        let language = LanguageModeSetting { mode: LanguageMode::Auto, primary: "el".into() };
        let calls = AtomicUsize::new(0);
        let result = transcribe_in_chosen_languages(&language, TranscriptionRequest::default(), "test", |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            std::future::ready(Ok(TranscriptionResult { text: "Привет".into(), detected_language: Some("ru".into()), ..Default::default() }))
        }).await.unwrap();
        assert_eq!(result.text, "Привет");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
