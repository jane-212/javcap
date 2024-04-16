local videos = {
    'STARS-804.mp4',
    'SONE-143.mp4',
    'FC2-PPV-1292936.mp4',
    'ABF-047.mp4',
}

local out = "out"

local function clear()
    local files = {
        out,
        'output',
        'logs',
        'other',
    }
    for _, file in pairs(files) do
        os.execute('rm -rf ' .. file)
    end
end

local function run()
    clear()
    os.execute('mkdir ' .. out)
    for _, video in pairs(videos) do
        local file = io.open(out .. '/' .. video, "w+")
        if file then
            file:write('hello')
        end
    end
    os.execute('cargo run')
end

local function help()
    print('run - clear and run the app')
    print('clear - clear the tmp dir')
end

if #arg ~= 1 then
    print('command error')
    os.exit(1)
end
local cmds = {
    ["run"] = run,
    ["clear"] = clear,
}
local cmd = cmds[arg[1]]
if cmd then
    cmd()
else
    print('cmd ' .. arg[1] .. ' not support')
    help()
end
