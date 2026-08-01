pub const Config = @import("Config.zig");
pub const App = @import("App.zig");
pub const Subdirs = @import("Subdirs.zig");

test {
    _ = @import("Config.zig");
    _ = @import("App.zig");
    _ = @import("Subdirs.zig");
}
