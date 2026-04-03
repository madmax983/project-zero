#!/usr/bin/perl
use strict;
use warnings;

my $file = 'src/layer1/designation.rs';
open my $in, '<', $file or die "Cannot open $file: $!";
my $content = do { local $/; <$in> };
close $in;

$content =~ s/fn get_valid_targets_for_tool\(world: &World,/fn get_valid_targets_for_tool(world: &mut World,/g;

open my $out, '>', $file or die "Cannot open $file for writing: $!";
print $out $content;
close $out;
