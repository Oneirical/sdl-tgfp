---@module "lib.consider.attack"
local combat = require "combat";
local world = require "world";

local considerations = {}

-- NOTE: As this is a motion ability, it could be interesting to add a motion-based
-- heuristic. It is more valuable to close in on escaping weak targets while oneself is strong,
-- while putting distance between oneself and strong foes has more value if one is weak.

for _, character in ipairs(world.characters_within(User.x, User.y, 1)) do
	if not combat.alliance_check(User, character) then
		table.insert(considerations, {
			arguments = { target = { x = character.x, y = character.y } },
			heuristics = {
				Heuristic.damage(
					character,
					Magnitude - character.stats.defense
				),
			}
		})
	end
end

return considerations

