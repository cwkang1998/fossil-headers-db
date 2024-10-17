WITH block_range AS (
    SELECT MIN(number) AS min_block, MAX(number) AS max_block
    FROM public.blockheaders
),
all_blocks AS (
    SELECT n AS block_number
    FROM generate_series(
        (SELECT min_block FROM block_range), 
        (SELECT max_block FROM block_range)
    ) n
),
missing_blocks AS (
    SELECT all_blocks.block_number
    FROM all_blocks
    LEFT JOIN public.blockheaders bh ON all_blocks.block_number = bh.number
    WHERE bh.number IS NULL
)
SELECT * FROM missing_blocks;
