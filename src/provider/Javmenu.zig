const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const provider = @import("root.zig");
const domain = @import("domain");
const infra = @import("infra");
const zq = @import("zigquery");
const utils = @import("utils");

const Self = @This();

alloc: Allocator,
io: Io,
client: infra.HttpClient,

pub fn init(alloc: Allocator, io: Io) !*Self {
    const client = infra.HttpClient.init(alloc, io, .{ .interval_ms = 1000, .retry = 3 });

    const self = try alloc.create(Self);
    errdefer alloc.destroy(self);

    self.* = .{
        .alloc = alloc,
        .io = io,
        .client = client,
    };

    return self;
}

pub fn deinit(self: *Self) void {
    self.client.deinit();
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
    var nfo = domain.Nfo.init(alloc);
    errdefer nfo.deinit();

    const show = try key.show(self.alloc);
    defer self.alloc.free(show);

    const url = try std.fmt.allocPrint(self.alloc, "https://javmenu.com/zh/{s}", .{show});
    defer self.alloc.free(url);

    const uri = try std.Uri.parse(url);
    var response = try self.client.fetch(self.alloc, .{
        .location = .{ .uri = uri },
    });
    defer response.deinit();
    if (response.status != .ok) return error.StatusNotOk;

    const body = response.body;
    var html = try zq.Document.initFromSlice(self.alloc, body);
    defer html.deinit();

    const titleSel = try html.find("h1 strong");
    const title = try utils.string.trimAll(alloc, try titleSel.text(), " \n\r\t");
    nfo.title = title;

    return nfo;
}
