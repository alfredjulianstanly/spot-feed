-- Messages in joints
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    joint_id UUID NOT NULL REFERENCES joints(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    content TEXT NOT NULL,
    message_type VARCHAR(20) DEFAULT 'text', -- 'text', 'image', 'audio', 'video'
    media_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_messages_joint ON messages(joint_id);
CREATE INDEX idx_messages_user ON messages(user_id);
CREATE INDEX idx_messages_created ON messages(joint_id, created_at DESC);

COMMENT ON TABLE messages IS 'Chat messages within joints';
COMMENT ON COLUMN messages.message_type IS 'text, image, audio, or video';
COMMENT ON COLUMN messages.media_url IS 'URL to media in R2 storage (for non-text messages)';
