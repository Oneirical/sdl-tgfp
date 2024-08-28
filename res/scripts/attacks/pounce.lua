---@module "lib.attack"
local combat = require "combat"
local world = require "world"

local target = world.character_at(Arguments.target.x, Arguments.target.y)
if target == nil then return end

-- TODO: see scratch.lua for info
-- if combat.alliance_check(User, target) and not combat.alliance_prompt() then return end

-- TODO: It could be interesting to have some helper functions like "adjacent_tiles" or "pick_adjacent" 
local adjacent_tiles = {
  { Arguments.target.x + 1, Arguments.target.y },
  { Arguments.target.x - 1, Arguments.target.y },
  { Arguments.target.x, Arguments.target.y + 1 },
  { Arguments.target.x, Arguments.target.y - 1 },
  { Arguments.target.x - 1, Arguments.target.y + 1 },
  { Arguments.target.x - 1, Arguments.target.y - 1 },
  { Arguments.target.x + 1, Arguments.target.y + 1 },
  { Arguments.target.x + 1, Arguments.target.y - 1 },
}

local filtered_tiles = {}
local count = 0

for _, tile in ipairs(adjacent_tiles) do
  -- TODO once again, "Wall" is not restrictive enough to exclude other solid tiles
		local tile_check = world.tile(tile[1], tile[2])
    if tile_check and tile_check ~= "Wall" then
        table.insert(filtered_tiles, tile)
        count = count + 1
    end
end

if count > 0 then
    local random_index = math.random(count)
    local random_tile = filtered_tiles[random_index]
    User.x = random_tile[1]
    User.y = random_tile[2]
else
    return -- No valid tiles left
end

-- Pounce's high momentum strike makes it easy to pierce.
local damage, pierce_failed = combat.apply_damage_with_pierce(1, Magnitude - target.stats.defense)

target.hp = target.hp - damage

local damage_messages = {
	"{self_Address} sprints and strikes {target_address}",
	"{self_Address} pounces onto {target_address}",
	"{self_Address} pummels {target_address} from the air",
}
local glance_messages = {
	"{self_Address} lightly taps {target_address}",
	"{self_Address} grazes {target_address} as {self_they} tumbles to the ground",
}
local failure_messages = {
	"{target_Address} sidesteps {self_address}'s pounce",
	"{self_Address} overshot their landing and missed {target_address}",
}

local function pick(table)
	return combat.format(User, target, table[math.random(#table)])
end

if pierce_failed then
	Console:combat_log(pick(glance_messages), Log.Glance)
elseif damage == 0 then
	Console:combat_log(pick(failure_messages), Log.Miss)
else
	Console:combat_log(pick(damage_messages), Log.Hit(damage))
end

return 12

