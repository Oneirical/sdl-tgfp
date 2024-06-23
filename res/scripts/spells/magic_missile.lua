local damage = math.max(magnitude - target.sheet:stats().resistance, 0)
target.hp -= damage
caster.sp -= level

local damage_messages = {
  "{self_Address}'s magic missile strikes {target_address}",
  "{self_Address} fires a magic missile at {target_address}",
  "{self_Address} conjures a magic missile, targetting {target_address}",
}
-- Shown when damage <= 1
local weak_messages = {
	"{self_Address}'s magic missile flies past {target_address}",
  "{self_Address}'s magic missile weakly glances against {target_address}",
  "{target_Address} narrowly dodges {self_Address}'s magic missile",
}
-- Shown when affinity is `Weak` and damage is 0.
local unskilled_messages = {
  "{self_Address}'s magic missile explodes mid-flight",
  "{self_Address} summons a misshapen magic missile, veering away from the target",
  "A misfired magic missile falls to the ground in front of {self_address}",
  "{self_Address} miscasts magic missile",
}

function pick(table)
  return target:replace_prefixed_nouns(
    "target_",
    caster:replace_prefixed_nouns(
      "self_",
      table[math.random(#table)]
    )
  )
end

local log = { Hit = {
  magnitude = magnitude,
  damage = damage,
}}

if damage == 0 and affinity:weak() and math.random(0, 1) == 1 then
  Console:combat_log(pick(unskilled_messages), log)
elseif damage <= 1 then
  Console:combat_log(pick(weak_messages), log)
else
  Console:combat_log(pick(damage_messages), log)
end
