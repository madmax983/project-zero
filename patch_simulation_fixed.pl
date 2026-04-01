#!/usr/bin/perl

use strict;
use warnings;

my $file = 'src/simulation.rs';
open my $in, '<', $file or die $!;
my @lines = <$in>;
close $in;

open my $out, '>', $file or die $!;
my $in_grafting_if = 0;
for my $line (@lines) {
    print $out $line;
    if ($line =~ /if !world\s*\.contains_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>\(\)/) {
        $in_grafting_if = 1;
    }

    if ($in_grafting_if && $line =~ /}/) {
        print $out "    if !world.contains_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>() {\n";
        print $out "        world.init_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>();\n";
        print $out "        world.init_resource::<Events<crate::layer3::market::quantum_famine::ExportDumpEvent>>();\n";
        print $out "    }\n";
        $in_grafting_if = 0;
    } elsif (!$in_grafting_if && $line =~ /world\.init_resource::<Events<crate::layer1::grafting::GraftBuildingEvent>>\(\);/ && $line !~ /if !world/) {
        print $out "        world.init_resource::<Events<crate::layer3::market::quantum_famine::MarketPanicEvent>>();\n";
        print $out "        world.init_resource::<Events<crate::layer3::market::quantum_famine::ExportDumpEvent>>();\n";
    }
}
close $out;
