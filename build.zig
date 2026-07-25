const std = @import("std");
const zon = @import("build.zig.zon");
const builtin = @import("builtin");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const strip = if (builtin.mode == .Debug) false else true;

    const exe = b.addExecutable(.{
        .name = "javcap",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
            .strip = strip,
        }),
    });
    b.installArtifact(exe);
    const exe_tests = b.addTest(.{
        .root_module = exe.root_module,
    });

    const mod = b.addModule("javcap", .{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
        .strip = strip,
    });
    exe.root_module.addImport("javcap", mod);
    const mod_tests = b.addTest(.{
        .root_module = mod,
    });

    const options = b.addOptions();
    options.addOption([]const u8, "name", "javcap");
    options.addOption([]const u8, "version", zon.version);
    mod.addOptions("options", options);

    const domain = b.createModule(.{
        .root_source_file = b.path("src/domain/root.zig"),
        .target = target,
        .optimize = optimize,
        .strip = strip,
    });
    mod.addImport("domain", domain);
    const domain_tests = b.addTest(.{
        .root_module = domain,
    });

    const media = b.createModule(.{
        .root_source_file = b.path("src/media/root.zig"),
        .target = target,
        .optimize = optimize,
        .strip = strip,
    });
    media.addImport("domain", domain);
    mod.addImport("media", media);
    const media_tests = b.addTest(.{
        .root_module = media,
    });

    const known_folders = b.dependency("known_folders", .{
        .target = target,
        .optimize = optimize,
    });
    mod.addImport("known-folders", known_folders.module("known-folders"));

    const mecha = b.dependency("mecha", .{
        .target = target,
        .optimize = optimize,
    });
    media.addImport("mecha", mecha.module("mecha"));

    const run_step = b.step("run", "Run the app");
    const run_cmd = b.addRunArtifact(exe);
    run_step.dependOn(&run_cmd.step);
    run_cmd.step.dependOn(b.getInstallStep());
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const test_step = b.step("test", "Run tests");
    const run_mod_tests = b.addRunArtifact(mod_tests);
    const run_exe_tests = b.addRunArtifact(exe_tests);
    const run_media_tests = b.addRunArtifact(media_tests);
    const run_domain_tests = b.addRunArtifact(domain_tests);
    test_step.dependOn(&run_mod_tests.step);
    test_step.dependOn(&run_exe_tests.step);
    test_step.dependOn(&run_media_tests.step);
    test_step.dependOn(&run_domain_tests.step);

    const check_step = b.step("check", "Check compile");
    check_step.dependOn(&run_cmd.step);
}
