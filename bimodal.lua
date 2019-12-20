local short
local long
local islong
local count = 1

function init(args)
	if table.getn(args) ~= 2
	then
		print('USAGE: wrk [arg]... -s.../bimodal.lua <url> <altpath> <alt‰>')
		os.exit(1)
	end

	short = wrk.format('GET')
	long = wrk.format('GET', args[1])

	local thresh = tonumber(args[2])
	islong = {}
	for i = 1, 1000
	do
		islong[i] = false
	end

	local seed = string.gsub(tostring(wrk.thread), 'userdata: 0x', '')
	math.randomseed(tonumber(seed, 16))
	for i = 1, thresh
	do
		local index
		repeat
			index = math.random(1000)
		until not islong[index]

		islong[index] = true
	end
end

function request()
	local req = short
	if islong[count]
	then
		req = long
	end

	count = count % 1000 + 1
	return req
end
