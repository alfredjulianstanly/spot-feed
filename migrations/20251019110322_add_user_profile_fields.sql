-- Add profile fields to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS display_name VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS profile_picture_url TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone_number VARCHAR(20);

COMMENT ON COLUMN users.display_name IS 'User display name (optional)';
COMMENT ON COLUMN users.profile_picture_url IS 'URL to user profile picture';
COMMENT ON COLUMN users.phone_number IS 'User phone number (optional)';
