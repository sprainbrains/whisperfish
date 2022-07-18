UPDATE attachments
    SET attachment_path =
        REPLACE(
            attachment_path,
            '.local/share/be.rubdos/harbour-whisperfish',
            '.local/share/harbour-whisperfish'
        );