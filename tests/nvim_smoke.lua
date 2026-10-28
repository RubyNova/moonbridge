-- Run with nvim -u NONE -i NONE --headless -l tests/nvim_smoke.lua.
-- Set NVIM_DAP_PATH if your nvim-dap plugin is installed somewhere else.

-- Load nvim-dap without loading your usual Neovim configuration.
local root = vim.fn.getcwd()
local plugin = vim.env.NVIM_DAP_PATH or (vim.env.HOME .. '/.local/share/nvim/lazy/nvim-dap')
vim.opt.runtimepath:append(plugin)
local dap = require('dap')

-- Create a small Lua file to debug.
local directory = vim.fn.tempname()
vim.fn.mkdir(directory, 'p')

local script = directory .. '/sample.lua'
vim.fn.writefile({
  'local answer = 41',
  'local data = { nested = { value = 9 } }',
  'answer = answer + 1',
  'print(answer)',
}, script)

local function wait_for(predicate, message)
  assert(vim.wait(10000, predicate, 10), message)
end

-- Send one debugger request and wait for its answer.
local function request(command, args)
  local done, response, failure = false, nil, nil

  dap.session():request(command, args, function(err, body)
    failure, response, done = err, body, true
  end)

  wait_for(function() return done end, 'Timed out: ' .. command)
  assert(not failure, vim.inspect(failure))

  return response
end

-- If a check fails, save the error so we can still clean up afterwards.
local ok, err = xpcall(function()
  dap.adapters['moonbridge-test'] = {
    type = 'executable',
    command = root .. '/target/debug/moonbridge',
  }

  -- Open the test file and put a breakpoint on line 3.
  vim.cmd.edit(vim.fn.fnameescape(script))
  vim.api.nvim_win_set_cursor(0, {3, 0})
  dap.set_breakpoint()

  dap.run({
    name = 'Moonbridge smoke test',
    type = 'moonbridge-test',
    request = 'launch',
    program = {
      lua = vim.env.LUA_EXECUTABLE or 'lua',
      file = script,
    },
    env = {LOCAL_LUA_DEBUGGER_VSCODE = '1'},
  })

  wait_for(function()
    local session = dap.session()
    local frame = session and session.current_frame

    return frame and frame.line == 3 and frame.scopes and frame.scopes[1].variables ~= nil
  end, 'Neovim did not load the stopped frame and local variables')

  -- Neovim should show the local answer variable with its value before line 3 runs.
  local frame = dap.session().current_frame
  local found = false

  for _, variable in ipairs(frame.scopes[1].variables) do
    if variable.name == 'answer' and variable.value == '41' then
      found = true
    end
  end

  assert(found, 'Neovim did not receive the local answer variable')

  -- Check that an expression can read a value inside a nested table.
  local evaluation = request('evaluate', {frameId = frame.id, expression = 'data.nested.value'})
  assert(evaluation.result == '9', vim.inspect(evaluation))

  -- Run line 3. The program should stop on line 4 with answer changed to 42.
  dap.step_over()

  wait_for(function()
    local session = dap.session()

    return session and session.current_frame and session.current_frame.line == 4
  end, 'Step over did not reach line 4')

  assert(request('evaluate', {expression = 'answer'}).result == '42')

  -- Finish running the program and wait for the debug session to close.
  dap.continue()
  wait_for(function() return dap.session() == nil end, 'Session did not terminate')
end, debug.traceback)

-- Close any session left open by a failed check, then remove the temporary file.
if dap.session() then
  dap.close()
end

vim.fn.delete(directory, 'rf')

if not ok then
  io.stderr:write(err .. '\n')
  vim.cmd('cquit 1')
else
  print('Moonbridge nvim-dap smoke test passed')
  vim.cmd('qa!')
end
