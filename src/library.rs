pub static DISCO_LIB: &str = "
function watch(command, err, linefn)
	return coroutine.create(function()
		local handle = io.popen(_)
		if not handle then return err end
		for line in handle:lines() do
			if linefn then line = linefn(line) end
			coroutine.yield(line)
		end
	end)
end
";
