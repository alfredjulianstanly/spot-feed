-- Create joints table without PostGIS (workaround)
-- Using separate lat/lon columns instead of GEOGRAPHY type

CREATE TABLE joints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joint_type VARCHAR(20) NOT NULL DEFAULT 'public',
    visibility VARCHAR(20) NOT NULL DEFAULT 'visible',
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    radius INTEGER NOT NULL DEFAULT 500,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Constraints
    CONSTRAINT valid_latitude CHECK (latitude >= -90 AND latitude <= 90),
    CONSTRAINT valid_longitude CHECK (longitude >= -180 AND longitude <= 180)
);

-- Indexes
CREATE INDEX idx_joints_creator ON joints(creator_id);
CREATE INDEX idx_joints_expires_at ON joints(expires_at);
CREATE INDEX idx_joints_location ON joints(latitude, longitude);
CREATE INDEX idx_joints_type ON joints(joint_type);

COMMENT ON TABLE joints IS 'Location-based groups that expire after 6 hours';
COMMENT ON COLUMN joints.latitude IS 'Latitude coordinate (-90 to 90)';
COMMENT ON COLUMN joints.longitude IS 'Longitude coordinate (-180 to 180)';
COMMENT ON COLUMN joints.radius IS 'Visibility radius in meters (default 500m)';
