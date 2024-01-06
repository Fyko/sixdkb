create extension if not exists pgcrypto;

-- MIT License - Copyright (c) 2023 Fabio Lima
create or replace function ksuid_pgcrypto_micros() returns text as $$
declare
	v_time timestamp with time zone := null;
	v_seconds numeric(50) := null;
	v_micros numeric(50)  := null;
	v_numeric numeric(50) := null;
	v_epoch numeric(50) = 1400000000; -- 2014-05-13T16:53:20Z
	v_payload bytea := null;
	v_base62 text := '';
	v_alphabet char array[62] := array[
		'0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
		'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
		'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 
		'U', 'V', 'W', 'X', 'Y', 'Z', 
		'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 
		'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
		'u', 'v', 'w', 'x', 'y', 'z'];
	i integer := 0;
begin

	-- Get the current time
	v_time := clock_timestamp();

	-- Extract the epoch seconds and microseconds
	v_seconds := EXTRACT(EPOCH FROM v_time) - v_epoch;
	v_micros  := MOD((EXTRACT(microseconds FROM v_time)::numeric(50)), 1e6::numeric(50));

	-- Generate a KSUID in a numeric variable
	v_numeric := (v_seconds * pow(2::numeric(50), 128))  -- 32 bits for seconds
		+ (v_micros  * pow(2::numeric(50), 108));        -- 20 bits for microseconds

	-- Add 108 random bits to it
	v_payload := gen_random_bytes(14);
	v_payload := set_byte(v_payload::bytea, 0, get_byte(v_payload, 0) >> 4);
	while i < 14 loop
		i := i + 1;
		v_numeric := v_numeric + (get_byte(v_payload, i - 1)::numeric(50) * pow(2::numeric(50), (14 - i) * 8));
	end loop;

	-- Encode it to base-62
	while v_numeric <> 0 loop
		v_base62 := v_base62 || v_alphabet[mod(v_numeric, 62) + 1];
		v_numeric := div(v_numeric, 62);
	end loop;
	v_base62 := reverse(v_base62);
	v_base62 := lpad(v_base62, 27, '0');

	return v_base62;
	
end $$ language plpgsql;

create or replace function generate_prefixed_ksuid(prefix text)
returns text as $$
declare
    ksuid text;
begin
    ksuid := ksuid_pgcrypto_micros();
    return prefix || '_' || ksuid;
end;
$$ language plpgsql;

create or replace function set_updated_at()
    returns trigger as
$$
begin
    NEW.updated_at = current_timestamp;
    return NEW;
end;
$$ language plpgsql;

create or replace function trigger_updated_at(tablename regclass)
    returns void as
$$
begin
    execute format('create trigger set_updated_at
        before update
        on %s
        for each row
        when (OLD is distinct from NEW)
    execute function set_updated_at();', tablename);
end;
$$ language plpgsql;

create collation case_insensitive (provider = icu, locale = 'und-u-ks-level2', deterministic = false);
