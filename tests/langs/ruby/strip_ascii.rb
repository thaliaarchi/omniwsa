puts "Ruby #{RUBY_VERSION}"
puts "ASCII whitespace characters for String#strip:"
(0..255).each do |c|
  s = c.chr(Encoding::ASCII_8BIT)
  both = "#{s}x#{s}"
  stripped = both.strip
  if stripped != both
    direction = if stripped == "x"
      "on both sides"
    elsif stripped == "x#{s}"
      "only on the left"
    elsif stripped == "#{s}x"
      "only on the right"
    else
      "BUG"
    end
    puts "0x#{c.to_s(16).rjust(2, '0')} (#{s.inspect}) is stripped #{direction}"
  end
end
