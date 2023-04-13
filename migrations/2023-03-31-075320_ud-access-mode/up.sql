-- This column was tinyint, and identified as Bool according to Diesel
ALTER TABLE recipients
    DROP COLUMN unidentified_access_mode;

-- 0 = Unknown
ALTER TABLE recipients
    ADD COLUMN unidentified_access_mode INTEGER NOT NULL DEFAULT 0;
