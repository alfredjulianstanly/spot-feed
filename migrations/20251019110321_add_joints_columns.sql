-- Add missing columns to joints table
ALTER TABLE joints ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE joints ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT TRUE NOT NULL;

-- Add index for active joints
CREATE INDEX IF NOT EXISTS idx_joints_active ON joints(is_active) WHERE is_active = true;

COMMENT ON COLUMN joints.description IS 'Optional description of the joint';
COMMENT ON COLUMN joints.is_active IS 'Whether the joint is currently active';
