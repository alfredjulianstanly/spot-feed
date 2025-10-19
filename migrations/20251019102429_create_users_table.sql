-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    profile_pic_url TEXT,
    phone VARCHAR(20),
    is_18_plus BOOLEAN NOT NULL DEFAULT FALSE,
    is_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);

-- Add comment for documentation
COMMENT ON TABLE users IS 'User accounts for Spot Feed';
COMMENT ON COLUMN users.is_18_plus IS 'Age verification - user confirmed they are 18+';
COMMENT ON COLUMN users.is_verified IS 'Email verification status via OTP';
