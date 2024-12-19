puts "Ruby #{RUBY_VERSION}"
puts "Whitespace characters for String#strip:"
(0..255).each do |byte|
  c = byte.chr
  both = "#{c}x#{c}"
  stripped = both.strip
  if stripped != both
    direction = if stripped == "x"
      "on both sides"
    elsif stripped == "x#{c}"
      "only on the left"
    elsif stripped == "#{c}x"
      "only on the right"
    else
      "BUG"
    end
    puts "0x#{byte.to_s(16).rjust(2, '0')} (#{byte.chr.inspect}) is stripped #{direction}"
  end
end
