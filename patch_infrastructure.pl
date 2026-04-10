#!/usr/bin/perl
use strict;
use warnings;
use File::Find;

my $dir = 'src/';

sub process_file {
    my $file = $File::Find::name;
    return unless -f $file && $file =~ /\.rs$/;

    open my $fh, '<', $file or die "Cannot open $file: $!";
    my $content = do { local $/; <$fh> };
    close $fh;

    my $modified = 0;

    my @modules = qw(building structure housing room_quality window parasitic_architecture spontaneous_architecture ruins construction);
    my @tests = qw(building_gate_test structure_fragile_tests structure_jury_rig_tests structure_maintenance_tests work_building_tests shift_integration_tests);

    foreach my $mod (@modules) {
        if ($content =~ s/crate::layer1::\b$mod\b(\b|:)/crate::layer1::infrastructure::$mod$1/g) { $modified = 1; }
        if ($file =~ /src\/layer1\/infrastructure\/.*\.rs$/) {
            if ($content =~ s/super::\b$mod\b(\b|:)/super::$mod$1/g) { $modified = 1; }
        }
    }

    foreach my $test (@tests) {
        if ($content =~ s/crate::layer1::\b$test\b(\b|:)/crate::layer1::infrastructure::$test$1/g) { $modified = 1; }
    }

    if ($modified) {
        open my $out, '>', $file or die "Cannot write $file: $!";
        print $out $content;
        close $out;
        print "Updated $file\n";
    }
}

find(\&process_file, $dir);
