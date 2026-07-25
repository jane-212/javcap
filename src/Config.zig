const std = @import("std");
const Allocator = std.mem.Allocator;
const options = @import("options");
const Io = std.Io;
const domain = @import("domain");
const storage = domain.storage;
const known_folders = @import("known-folders");

pause_after_finish: bool,
sources: []Source,

pub const Source = struct {
    type: storage.Type,
    from: []const u8,
    to: []const u8,
    exts: []const []const u8,

    fn validate(self: *const Source, context: *Context) !void {
        if (!std.fs.path.isAbsolute(self.from)) {
            context.validate.field = try context.alloc.dupe(u8, "from");
            context.validate.value = try context.alloc.dupe(u8, self.from);
            context.validate.message = try context.alloc.dupe(u8, "路径必须是绝对路径");
            return error.ValidateFailed;
        }
        if (!std.fs.path.isAbsolute(self.to)) {
            context.validate.field = try context.alloc.dupe(u8, "to");
            context.validate.value = try context.alloc.dupe(u8, self.to);
            context.validate.message = try context.alloc.dupe(u8, "路径必须是绝对路径");
            return error.ValidateFailed;
        }
    }
};

const Self = @This();

pub fn init(alloc: Allocator, io: Io, context: *Context, env: *const std.process.Environ.Map) !Parsed(Self) {
    const path = try configPath(alloc, io, env);
    defer alloc.free(path);

    const file = Io.Dir.openFileAbsolute(io, path, .{}) catch |err| switch (err) {
        error.FileNotFound => {
            try generateDefaultConfig(alloc, io, env);
            return error.ConfigNotFound;
        },
        else => return err,
    };
    defer file.close(io);

    var buffer: [4096]u8 = undefined;
    var fileReader = file.reader(io, &buffer);
    const reader = &fileReader.interface;

    const configContent = try reader.allocRemaining(alloc, .unlimited);
    defer alloc.free(configContent);

    const configContentZ = try alloc.dupeSentinel(u8, configContent, 0);
    defer alloc.free(configContentZ);

    const config = try std.zon.parse.fromSliceAlloc(Self, alloc, configContentZ, null, .{});
    errdefer std.zon.parse.free(alloc, config);

    try config.validate(context);

    return .{
        .alloc = alloc,
        .value = config,
    };
}

pub fn Parsed(comptime T: type) type {
    return struct {
        alloc: Allocator,
        value: T,

        pub fn deinit(self: *@This()) void {
            std.zon.parse.free(self.alloc, self.value);
            self.* = undefined;
        }
    };
}

pub const Context = struct {
    alloc: Allocator,
    validate: ValidateContext = .{},

    pub fn init(alloc: Allocator) Context {
        return .{
            .alloc = alloc,
        };
    }

    pub fn deinit(self: *Context) void {
        if (self.validate.field) |field| self.alloc.free(field);
        if (self.validate.value) |value| self.alloc.free(value);
        if (self.validate.message) |message| self.alloc.free(message);
        self.* = undefined;
    }
};

pub const ValidateContext = struct {
    field: ?[]const u8 = null,
    value: ?[]const u8 = null,
    message: ?[]const u8 = null,

    pub fn format(self: *ValidateContext, writer: *Io.Writer) !void {
        try writer.print("{s} => {s}: {s}\n", .{ self.message.?, self.field.?, self.value.? });
        try writer.flush();
    }
};

fn validate(self: *const Self, context: *Context) !void {
    for (self.sources) |source| try source.validate(context);
}

fn generateDefaultConfig(alloc: Allocator, io: Io, env: *const std.process.Environ.Map) !void {
    const defaultConfig = @embedFile("config.zon");
    const dir = try configDir(alloc, io, env);
    defer alloc.free(dir);

    const cwd = Io.Dir.cwd();
    try cwd.createDirPath(io, dir);

    const path = try configPath(alloc, io, env);
    defer alloc.free(path);

    const file = try Io.Dir.createFileAbsolute(io, path, .{});
    defer file.close(io);

    var buffer: [4096]u8 = undefined;
    var fileWriter = file.writer(io, &buffer);
    const writer = &fileWriter.interface;

    try writer.writeAll(defaultConfig);
    try writer.flush();
}

fn configPath(alloc: Allocator, io: Io, env: *const std.process.Environ.Map) ![]const u8 {
    const dir = try configDir(alloc, io, env);
    defer alloc.free(dir);

    const path = try std.fs.path.join(alloc, &.{
        dir,
        "config.zon",
    });
    errdefer alloc.free(path);

    return path;
}

fn configDir(alloc: Allocator, io: Io, env: *const std.process.Environ.Map) ![]const u8 {
    const root = try known_folders.getPath(io, alloc, env, .local_configuration) orelse return error.ConfigHomeNotFound;
    defer alloc.free(root);

    const dir = try std.fs.path.join(alloc, &.{
        root,
        options.name,
    });
    errdefer alloc.free(dir);

    return dir;
}
