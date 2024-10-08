local device_key = KEYS[1]
local fd_key = KEYS[2]
local device_fd_key = KEYS[3]

-- Check if device exists
if redis.call("EXISTS", device_key) == 0 then
    return redis.error_reply("Device not found")
end

-- Check if flight data already exists
if redis.call("EXISTS", fd_key) == 1 then
    return redis.error_reply("Flight data already exists")
end

-- Check current dataset
local dataset_count = redis.call("HGET", device_key, "dataset_count")
local device_id = redis.call("HGET", device_key, "id")
local dataset_id = device_id .. ":" .. dataset_count
local dataset_key = "dataset:" .. dataset_id
local dataset_fd_count = tonumber(redis.call("HGET", dataset_key, "count"))
local dataset_limit = tonumber(redis.call("HGET", dataset_key, "limit"))

-- Create or update dataset
-- NOTE: the > case happens if dataset limit was decreased during operations
if dataset_fd_count >= dataset_limit then
    -- New dataset
    dataset_count = tonumber(redis.call("HINCRBY", device_key, "dataset_count", 1))
    dataset_id = device_id .. ":" .. dataset_count
    dataset_key = "dataset:" .. dataset_id
    redis.call("HSET", dataset_key, "id", dataset_id)
    redis.call("HSET", dataset_key, "device", device_key)
    redis.call("HSET", dataset_key, "limit", dataset_limit)
    redis.call("HSET", dataset_key, "count", 1)
else
    -- Existing dataset
    redis.call("HINCRBY", dataset_key, "count", 1)
end

-- Save flight data
local flight_data_timestamp = ARGV[3]
redis.call("HSET", fd_key, "id", ARGV[1])
redis.call("HSET", fd_key, "signature", ARGV[2])
redis.call("HSET", fd_key, "timestamp", ARGV[3])
redis.call("HSET", fd_key, "localization", ARGV[4])
redis.call("HSET", fd_key, "payload", ARGV[5])
redis.call("HSET", fd_key, "dataset", dataset_key)

-- Add flight data to dataset
local dataset_fd_key = "dataset_flight_data:" .. dataset_id
redis.call("ZADD", dataset_fd_key, flight_data_timestamp, fd_key)

-- Add flight data to device flight data set
redis.call("ZADD", device_fd_key, flight_data_timestamp, fd_key)

return dataset_key