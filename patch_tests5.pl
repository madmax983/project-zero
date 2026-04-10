#!/usr/bin/perl
use strict;
use warnings;

my $file = 'src/layer1/infrastructure/mod.rs';
open my $fh, '<', $file or die "Cannot open $file: $!";
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/    use super::\*\;\n//g;
$content =~ s/    use crate::layer1::\*\; \n//g;

open my $out, '>', $file or die "Cannot write $file: $!";
print $out $content;
close $out;
