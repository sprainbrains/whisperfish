UPDATE attachments
    SET attachment_path =
        REPLACE(
            attachment_path,
            '.local/share/harbour-whisperfish',
            '.local/share/be.rubdos/harbour-whisperfish'
        );