require("combat")

local considerations = ...

for i, character in ipairs(nearby_characters) do
	if not alliance_check(user, character) then
		considerations:push(Heuristic:damage(
			character,
			magnitude - character.stats.defense
		))
	end
end

return considerations
