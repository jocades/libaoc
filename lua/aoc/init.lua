local cmd = require("aoc.cmd")({ "aoc" })
local log = require("aoc.logger")()

local M = {}

function M.get()
  cmd:arg("get")
  cmd:arg("-b")

  cmd:spawn(function(p)
    if p.code ~= 0 then
      log.error(p.stderr)
      return
    end
  end)
end

local user_cmd_args = {
  "y",
  "year",
  "d",
  "day",
  "p",
  "part",
  "b",
  "build",
}

local function parse(args)
  local opts = {}
  for _, arg in ipairs(args.fargs) do
    local k, v = unpack(vim.split(arg, "="))
    if vim.tbl_contains(user_cmd_args, k) then
      opts[k] = v
    else
      return log.warn(("Unknown option: %s"):format(k))
    end
  end
  return opts
end

function M.setup()
  vim.api.nvim_create_user_command("AocGet", function(args)
    M.get()
  end, { nargs = "*" })
end

return M
