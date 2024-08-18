---@module "lib.spell"
local combat = require "combat"
local world = require "world"

-- Prompt user for arguments if they have not been provided
if Arguments == nil then
	Arguments = {
		target = world.target(User.x, User.y, Parameters.range)
	}
end

User.sp = User.sp - Level

local function format(s)
	return Arguments.target:replace_prefixed_nouns(
		"target_",
		User:replace_prefixed_nouns(
			"self_",
			s
		)
	)
end

if not combat.alliance_check(User, Arguments.target)
	and Affinity:magnitude(Parameters.magnitude) - Arguments.target.stats.resistance <= 0
then
	local log = { type = "Miss" }
	Console:combat_log(format("{target_Address} resisted {self_address}'s swap."), log)
else
	local cx, cy = User.x, User.y
	User.x = Arguments.target.x
	User.y = Arguments.target.y
	Arguments.target.x = cx
	Arguments.target.y = cy

	local log = { type = "Success" }
	Console:combat_log(format("{self_Address} swapped positions with {target_address}."), log)
end

return Parameters.cast_time
