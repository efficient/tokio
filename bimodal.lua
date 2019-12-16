local short
local long
local thresh

function init(args)
	if table.getn(args) ~= 2
	then
		print('USAGE: wrk [arg]... -s.../bimodal.lua <url> <altpath> <alt%>')
		os.exit(1)
	end

	short = wrk.format('GET')
	long = wrk.format('GET', args[1])
	thresh = tonumber(args[2])

	local seed = string.gsub(tostring(wrk.thread), 'userdata: 0x', '')
	math.randomseed(tonumber(seed, 16))
end

function request()
	if math.random(100) <= thresh
	then
		return long
	else
		return short
	end
end
