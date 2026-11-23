//! Windows-only repair for screenshot producers that publish PNG without a
//! native bitmap flavor. Office/WPS mixed image+text payloads never enter this
//! path because any text, HTML, RTF, or file flavor rejects the candidate.

use super::guard::WritebackGuard;
use super::storage::ImageStore;
use crate::core::Result;
use crate::db::models::ClipboardItem;

/// Inspect the current clipboard before the watcher reads it.
pub fn image_repair_metadata() -> (bool, Option<u32>) {
    #[cfg(target_os = "windows")]
    {
        let snapshot = windows::inspect_formats();
        let candidate = windows::should_repair(&snapshot);
        (candidate, snapshot.sequence)
    }

    #[cfg(not(target_os = "windows"))]
    {
        (false, None)
    }
}

/// Rewrite a captured pure image only when the setting is enabled and the
/// clipboard sequence still matches the captured sequence.
pub fn repair_after_persist(
    enabled: bool,
    store: &ImageStore,
    guard: &WritebackGuard,
    item: &ClipboardItem,
    repair_candidate: bool,
    source_sequence: Option<u32>,
) -> Result<()> {
    if !enabled || !repair_candidate {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        windows::rewrite_if_still_png_only(store, guard, item, source_sequence)?;
    }

    let _ = (store, guard, item, source_sequence);
    Ok(())
}

#[cfg(target_os = "windows")]
mod windows {
    use clipboard_win::formats;
    use clipboard_win::raw;

    use crate::clipboard::write_to_clipboard;
    use crate::core::{AppError, Result};
    use crate::db::models::ClipboardItem;

    use super::super::guard::WritebackGuard;
    use super::super::storage::ImageStore;

    const HTML_FORMAT: &str = "HTML Format";
    const PNG_FORMAT: &str = "PNG";
    const RTF_FORMAT: &str = "Rich Text Format";

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub(super) struct FormatSnapshot {
        pub png: bool,
        pub bitmap: bool,
        pub dib: bool,
        pub dibv5: bool,
        pub text: bool,
        pub html: bool,
        pub rtf: bool,
        pub files: bool,
        pub sequence: Option<u32>,
    }

    pub(super) fn should_repair(snapshot: &FormatSnapshot) -> bool {
        snapshot.png
            && !snapshot.bitmap
            && !snapshot.dib
            && !snapshot.dibv5
            && !snapshot.text
            && !snapshot.html
            && !snapshot.rtf
            && !snapshot.files
    }

    pub(super) fn inspect_formats() -> FormatSnapshot {
        FormatSnapshot {
            png: custom_format_available(PNG_FORMAT),
            bitmap: raw::is_format_avail(formats::CF_BITMAP),
            dib: raw::is_format_avail(formats::CF_DIB),
            dibv5: raw::is_format_avail(formats::CF_DIBV5),
            text: raw::is_format_avail(formats::CF_UNICODETEXT),
            html: custom_format_available(HTML_FORMAT),
            rtf: custom_format_available(RTF_FORMAT),
            files: raw::is_format_avail(formats::CF_HDROP),
            sequence: raw::seq_num().map(|value| value.get()),
        }
    }

    pub(super) fn rewrite_if_still_png_only(
        store: &ImageStore,
        guard: &WritebackGuard,
        item: &ClipboardItem,
        captured_sequence: Option<u32>,
    ) -> Result<()> {
        let Some(captured_sequence) = captured_sequence else {
            return Ok(());
        };
        let current = inspect_formats();
        if current.sequence != Some(captured_sequence) || !should_repair(&current) {
            return Ok(());
        }

        write_to_clipboard(store, guard, item, false)?;

        let repaired = inspect_formats();
        if repaired.sequence == current.sequence
            || (!repaired.bitmap && !repaired.dib && !repaired.dibv5)
        {
            return Err(AppError::Clipboard(
                "clipboard repair completed without a standard bitmap format".to_owned(),
            ));
        }

        log::info!(
            "repaired PNG-only clipboard image with standard bitmap formats (sequence {} -> {:?})",
            captured_sequence,
            repaired.sequence
        );
        Ok(())
    }

    fn custom_format_available(name: &str) -> bool {
        raw::is_format_avail(raw::register_format(name).map_or(0, |format| format.get()))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn png_only_without_text_is_a_repair_candidate() {
            let snapshot = FormatSnapshot {
                png: true,
                sequence: Some(10),
                ..FormatSnapshot::default()
            };

            assert!(should_repair(&snapshot));
        }

        #[test]
        fn native_bitmap_or_files_disqualify_repair() {
            for snapshot in [
                FormatSnapshot {
                    png: true,
                    bitmap: true,
                    ..FormatSnapshot::default()
                },
                FormatSnapshot {
                    png: true,
                    dib: true,
                    ..FormatSnapshot::default()
                },
                FormatSnapshot {
                    png: true,
                    dibv5: true,
                    ..FormatSnapshot::default()
                },
                FormatSnapshot {
                    png: true,
                    files: true,
                    ..FormatSnapshot::default()
                },
            ] {
                assert!(!should_repair(&snapshot));
            }
        }

        #[test]
        fn office_and_rich_text_formats_are_never_repaired() {
            for snapshot in [
                FormatSnapshot {
                    png: true,
                    text: true,
                    ..FormatSnapshot::default()
                },
                FormatSnapshot {
                    png: true,
                    html: true,
                    ..FormatSnapshot::default()
                },
                FormatSnapshot {
                    png: true,
                    rtf: true,
                    ..FormatSnapshot::default()
                },
            ] {
                assert!(!should_repair(&snapshot));
            }
        }
    }
}
