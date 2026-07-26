const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const provider = @import("root.zig");
const domain = @import("domain");

const Self = @This();

alloc: Allocator,
io: Io,

pub fn init(alloc: Allocator, io: Io) !*Self {
    const self = try alloc.create(Self);
    self.* = .{
        .alloc = alloc,
        .io = io,
    };
    return self;
}

pub fn deinit(self: *Self) void {
    self.alloc.destroy(self);
    self.* = undefined;
}

pub fn asProvider(self: *Self) provider.Provider {
    const Impl = struct {
        fn searchInner(
            ptr: *anyopaque,
            alloc: Allocator,
            key: domain.jav.Key,
        ) !domain.Nfo {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return try s.search(alloc, key);
        }

        fn deinitInner(
            ptr: *anyopaque,
        ) void {
            const s: *Self = @ptrCast(@alignCast(ptr));
            s.deinit();
        }

        const vtable = provider.VTable{
            .search = searchInner,
            .deinit = deinitInner,
        };
    };

    return .{
        .ptr = self,
        .vtable = &Impl.vtable,
    };
}

pub fn search(self: *Self, alloc: Allocator, key: domain.jav.Key) !domain.Nfo {
    _ = self;

    var nfo = domain.Nfo.init(alloc);
    errdefer nfo.deinit();

    nfo.title = switch (key) {
        .jav => |jav| try std.fmt.allocPrint(alloc, "file key: {s}-{s}", .{ jav.id, jav.number }),
        .fc2 => |fc2| try std.fmt.allocPrint(alloc, "file key: FC2-{s}", .{fc2}),
        .normal => |normal| try std.fmt.allocPrint(alloc, "{s}", .{normal}),
    };

    return nfo;
}
