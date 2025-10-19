-- Joint members (who's in which joint)
CREATE TABLE joint_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    joint_id UUID NOT NULL REFERENCES joints(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'member', -- 'creator', 'moderator', 'member'
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(joint_id, user_id)
);

-- Indexes
CREATE INDEX idx_joint_members_joint ON joint_members(joint_id);
CREATE INDEX idx_joint_members_user ON joint_members(user_id);
CREATE INDEX idx_joint_members_role ON joint_members(joint_id, role);

COMMENT ON TABLE joint_members IS 'Tracks which users are in which joints';
COMMENT ON COLUMN joint_members.role IS 'creator, moderator, or member';
