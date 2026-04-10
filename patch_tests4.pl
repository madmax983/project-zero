#!/usr/bin/perl
use strict;
use warnings;

my $file = 'src/layer1/mod.rs';
open my $fh, '<', $file or die "Cannot open $file: $!";
my $content = do { local $/; <$fh> };
close $fh;

# Fix empty lines after doc comments
$content =~ s/(\/\/\/[^\n]*\n)\n+/$1/g;

open my $out, '>', $file or die "Cannot write $file: $!";
print $out $content;
close $out;
