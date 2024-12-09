---@param init_args string[]
---@param opts? { flag_prefix?: string }
local function Command(init_args, opts)
  opts = opts or {}

  ---@class Command
  local cmd = {
    _args = init_args,
    ---@type table<string,string>
    _flags = {},
    _vars = {},
  }

  ---@param flag string
  local function mkflag(flag)
    return (opts.flag_prefix or "-") .. flag
  end

  ---Add an argument.
  ---@param val string
  function cmd:arg(val)
    table.insert(self._args, val)
  end

  ---Set an option.
  ---@param key string
  ---@param val? string
  function cmd:opt(key, val)
    if not val then
      table.insert(self._args, mkflag(key))
      return
    end
    self._flags[mkflag(key)] = val
  end

  ---Set an option only if a value is provided.
  ---@param key string
  ---@param val? string
  function cmd:optif(key, val)
    if val then
      self:opt(key, val)
    end
  end

  ---@param flag string
  function cmd:get(flag)
    return self._flags[mkflag(flag)]
  end

  ---@param flag string
  function cmd:has(flag)
    return self._flags[mkflag(flag)] ~= nil
  end

  function cmd:build()
    for k, v in pairs(self._flags) do
      table.insert(self._args, k)
      table.insert(self._args, v)
    end
    return self._args
  end

  local function env()
    return { __NVIM_AOC = vim.api.nvim_buf_get_name(0) }
  end

  ---@param callback fun(p: vim.SystemCompleted)
  function cmd:spawn(callback)
    return vim.system(self:build(), { text = true, env = env() }, callback)
  end

  return cmd
end

return Command
