
print("ECS example")

-- Example of a global entity
-- print(tostring(example_entity))

local camera_position = Vec3(0.0, 0.0, 0.0);
local square_entity_id = nil
local speed = 10.0

function on_init()
    print("Lua: on_init()")
    square_entity_id = proxy:create_square_lazy(0.5, 0.5, 'red')
    print("create_square_lazy " .. tostring(square_entity_id))
end

function on_update(delta_time)
    -- print("Lua: on_update(" .. tostring(delta_time) .. ")")
    -- print(tostring(proxy))

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
        local x, y = 0.0, 0.0
        
        -- Left
        if proxy:is_key_pressed(virtual_key_code.Left) then
             x = x - 1.0
        end

        -- Right
        if proxy:is_key_pressed(virtual_key_code.Right) then
             x = x + 1.0
        end

        -- Up
        if proxy:is_key_pressed(virtual_key_code.Up) then
            y = y + 1.0
        end

        -- Down
        if proxy:is_key_pressed(virtual_key_code.Down) then
            y = y - 1.0
        end

        camera_position = camera_position + Vec3(x, y, 0.0) * speed * delta_time
        proxy:set_camera_eye(camera_entity_id, camera_position)
    end
end
