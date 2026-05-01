mod tests {
    use amigo_audio_mixer::AudioMixFrame;

    use super::{AudioOutputBackendService, MAX_BUFFERED_SAMPLES};

    #[test]
    fn queues_audio_frames_without_starting_stream() {
        let service = AudioOutputBackendService::default();
        service.enqueue_mix_frame(&AudioMixFrame {
            sample_rate: 44_100,
            samples: vec![0.1, -0.2, 0.3, -0.4],
            sources: vec!["jump".to_owned()],
        });

        let snapshot = service.snapshot();
        assert_eq!(snapshot.backend_name, "system-audio");
        assert!(!snapshot.started);
        assert_eq!(snapshot.buffered_samples, 4);
    }

    #[test]
    fn trims_queued_samples_to_reasonable_limit() {
        let service = AudioOutputBackendService::default();
        service.enqueue_mix_frame(&AudioMixFrame {
            sample_rate: 44_100,
            samples: vec![0.0; MAX_BUFFERED_SAMPLES + 1024],
            sources: vec!["overflow".to_owned()],
        });

        let snapshot = service.snapshot();
        assert_eq!(snapshot.buffered_samples, MAX_BUFFERED_SAMPLES);
    }
}
