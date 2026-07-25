const std = @import("std");
const javcap = @import("javcap");
const Config = javcap.Config;

pub fn main(init: std.process.Init) !void {
    const alloc = init.gpa;
    const io = init.io;

    var context = Config.Context.init(alloc);
    defer context.deinit();
    var configRaw = Config.init(alloc, io, "jane", &context) catch |err| {
        switch (err) {
            error.ValidateFailed => {
                var buffer: [4096]u8 = undefined;
                var stderr = std.Io.File.stderr().writer(io, &buffer);
                try context.validate.format(&stderr.interface);
            },
            else => {},
        }

        return err;
    };
    defer configRaw.deinit();

    const config = configRaw.value;
    for (config.sources) |source| std.debug.print("{s}\n", .{source.from});
}
