import re

with open('src/app/mod.rs', 'r') as f:
    mod_rs = f.read()

# Extract tests
start_idx = mod_rs.find("#[cfg(test)]\nmod tests {")
if start_idx != -1:
    brace_count = 0
    end_idx = -1
    for i in range(start_idx, len(mod_rs)):
        if mod_rs[i] == '{': brace_count += 1
        elif mod_rs[i] == '}':
            brace_count -= 1
            if brace_count == 0:
                end_idx = i + 1
                break
    
    test_block = mod_rs[start_idx:end_idx]
    
    # Strip `mod tests { ... }` wrappers to put in tests.rs
    test_content = re.sub(r'^#\[cfg\(test\)\]\s*mod tests\s*\{', '', test_block)
    test_content = test_content[:-1] # remove last brace
    
    with open('src/app/tests.rs', 'w') as f:
        f.write("use super::*;\n" + test_content)
        
    mod_rs = mod_rs[:start_idx] + "\n#[cfg(test)]\nmod tests;\n" + mod_rs[end_idx:]

with open('src/app/mod.rs', 'w') as f:
    f.write(mod_rs)
