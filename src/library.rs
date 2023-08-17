pub static DISCO_LIB: &str = "
function watch(command, err)
	return coroutine.create(function()
		local handle = io.popen(_)
		if not handle then return err end
		for line in handle:lines() do
			coroutine.yield(line)
		end
	end)
end
";
