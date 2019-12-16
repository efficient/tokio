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
	math.randomseed(os.time())
end

function request()
	if math.random(100) <= thresh
	then
		return long
	else
		return short
	end
end
