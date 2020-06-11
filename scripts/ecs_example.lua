
print("ECS example")

-- Example of a global entity
-- print(tostring(example_entity))

local camera_position = Vec3(0.0, 0.0, 0.0);
local square_entity_id = nil
local speed = 0.1

function on_init()
    print("Lua: on_init()")
    square_entity_id = proxy:create_square_lazy('red')
    print("create_square_lazy " .. tostring(square_entity_id))
end

function on_update(delta_time)
    print("Lua: on_update(" .. tostring(delta_time) .. ")")
    print(tostring(proxy))

    -- get transform component
    -- local transform = proxy:get_transform(example_entity)
    -- if transform then
    --     print("Transform { " .. tostring(transform:get_position()) .. " }")
    -- else
    --     print("No transform found for " .. tostring(example_entity))
    -- end

    -- Move camera around
    local camera_entity_id = proxy:get_current_camera()
    if camera_entity_id then
        camera_position = camera_position + Vec3(-1.0, 0.0, 0.0) * speed * delta_time
        proxy:set_camera_eye(camera_entity_id, camera_position)
    end
end
