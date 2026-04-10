#!/usr/bin/perl
use strict;
use warnings;

my $file = 'src/layer1/mod.rs';
open my $fh, '<', $file or die "Cannot open $file: $!";
my $content = do { local $/; <$fh> };
close $fh;

my @modules = qw(building structure housing room_quality window parasitic_architecture spontaneous_architecture ruins construction);
foreach my $mod (@modules) {
    $content =~ s/pub mod $mod;\n//g;
}

$content =~ s/mod building_gate_test;\n//g;
$content =~ s/mod structure_fragile_tests;\n//g;
$content =~ s/mod structure_jury_rig_tests;\n//g;
$content =~ s/mod structure_maintenance_tests;\n//g;
$content =~ s/mod work_building_tests;\n//g;
$content =~ s/mod shift_integration_tests;\n//g;

if ($content !~ /pub mod infrastructure;/) {
    $content .= "\npub mod infrastructure;\n";
    $content .= "pub use infrastructure::*;\n";
}

open my $out, '>', $file or die "Cannot write $file: $!";
print $out $content;
close $out;
print "Updated $file\n";
