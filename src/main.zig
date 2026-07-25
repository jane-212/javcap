const std = @import("std");
const javcap = @import("javcap");
const Config = javcap.Config;

pub fn main(init: std.process.Init) !void {
    const alloc = init.gpa;
    const io = init.io;

    var context: Config.ValidateContext = .{
        .alloc = alloc,
    };
    defer context.deinit();
    var configRaw = Config.init(alloc, io, "jane", &context) catch |err| {
        switch (err) {
            error.FromPathShouldBeAbsolute,
            error.ToPathShouldBeAbsolute,
            => std.debug.print("{s} -> {s}: {s}\n", .{ context.message.?, context.field.?, context.value.? }),
            else => {},
        }

        return err;
    };
    defer configRaw.deinit();

    const config = configRaw.value;
    for (config.sources) |source| std.debug.print("{s}\n", .{source.from});
}
