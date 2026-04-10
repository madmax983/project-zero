#!/usr/bin/perl
use strict;
use warnings;

my @files = qw(
    src/layer1/infrastructure/building_gate_test.rs
    src/layer1/infrastructure/structure_fragile_tests.rs
    src/layer1/infrastructure/structure_jury_rig_tests.rs
    src/layer1/infrastructure/structure_maintenance_tests.rs
    src/layer1/infrastructure/work_building_tests.rs
    src/layer1/infrastructure/shift_integration_tests.rs
);

foreach my $file (@files) {
    open my $fh, '<', $file or die "Cannot open $file: $!";
    my $content = do { local $/; <$fh> };
    close $fh;

    # remove mod tests { ... }
    $content =~ s/^\#\[cfg\(test\)\]\nmod tests \{\n//g;
    $content =~ s/^\s*use super::\*;\n//mg;
    $content =~ s/^\s*use crate::layer1::\*;\n//mg;

    # Check if there is a closing bracket at the end and remove it
    $content =~ s/\n\}\n*$//;

    open my $out, '>', $file or die "Cannot write $file: $!";
    print $out $content;
    close $out;
}
