use super::*;

pub(crate) fn process_audio_command(
    command: AudioCommand,
    audio_state_service: &AudioStateService,
    dev_console_state: &DevConsoleState,
) {
    audio_state_service.record_processed_command(command.clone());
    audio_state_service.queue_runtime_command(command.clone());

    match command {
        AudioCommand::PlayOnce { clip } => {
            dev_console_state.write_line(format!("audio play once `{}`", clip.as_str()));
        }
        AudioCommand::StartSource { source, clip } => {
            audio_state_service.start_source(source.clone(), clip.clone());
            dev_console_state.write_line(format!(
                "audio start source `{}` -> `{}`",
                source.as_str(),
                clip.as_str()
            ));
        }
        AudioCommand::StopSource { source } => {
            let _ = audio_state_service.stop_source(source.as_str());
            dev_console_state.write_line(format!("audio stop source `{}`", source.as_str()));
        }
        AudioCommand::SetParam {
            source,
            param,
            value,
        } => {
            if audio_state_service.set_param(source.as_str(), param.clone(), value) {
                dev_console_state.write_line(format!(
                    "audio set param `{}` for `{}` = {}",
                    param,
                    source.as_str(),
                    value
                ));
            }
        }
        AudioCommand::SetVolume { bus, value } => {
            if audio_state_service.set_volume(&bus, value) {
                dev_console_state.write_line(format!(
                    "audio set bus volume `{bus}` = {}",
                    value.clamp(0.0, 1.0)
                ));
            }
        }
        AudioCommand::SetMasterVolume { value } => {
            if audio_state_service.set_master_volume(value) {
                dev_console_state.write_line(format!(
                    "audio set master volume = {}",
                    value.clamp(0.0, 1.0)
                ));
            }
        }
    }
}
