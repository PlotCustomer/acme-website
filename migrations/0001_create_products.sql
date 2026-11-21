CREATE TABLE products (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    slug text NOT NULL UNIQUE,
    name text NOT NULL,
    summary text NOT NULL,
    description text NOT NULL,
    product_type text NOT NULL CHECK (
        product_type IN ('everyday-goods', 'explosives-weapons', 'vehicles-gadgets')
    ),
    price_cents bigint NOT NULL CHECK (price_cents >= 0),
    currency char(3) NOT NULL DEFAULT 'USD',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX products_product_type_idx ON products (product_type);
CREATE INDEX products_name_idx ON products (name);

INSERT INTO products (
    slug,
    name,
    summary,
    description,
    product_type,
    price_cents,
    currency
) VALUES
    (
        'stoneware-pour-over-set',
        'Stoneware Pour-Over Set',
        'A hand-glazed dripper and carafe for a slow, even morning brew.',
        'The Stoneware Pour-Over Set pairs a ribbed ceramic dripper with a 600 ml carafe. The glazed interior keeps the brew temperature steady, while the wide handle stays comfortable through the pour. Dishwasher safe, and made to sit on an open kitchen shelf.',
        'everyday-goods',
        4800,
        'USD'
    ),
    (
        'linen-throw-blanket',
        'Linen Throw Blanket',
        'A breathable, stonewashed linen throw that softens with every wash.',
        'Woven from European flax and finished with a hand-knotted fringe, this throw is light enough for summer evenings and warm enough for a reading chair in winter. The stonewashed finish gives it a relaxed drape from the first use.',
        'everyday-goods',
        7200,
        'USD'
    ),
    (
        'copper-kitchen-scale',
        'Copper Kitchen Scale',
        'A compact countertop scale with a brushed copper face and precise gram readings.',
        'Reads up to five kilograms in one-gram steps, with a tare function for weighing straight into a mixing bowl. The low-profile body stores upright, and the copper face wipes clean without fuss. Two AAA batteries included.',
        'everyday-goods',
        3950,
        'USD'
    ),
    (
        'industrial-safety-flare-kit',
        'Industrial Safety Flare Kit',
        'A sealed roadside kit with high-visibility flares and a weatherproof storage case.',
        'Built for worksites, fleet vehicles, and remote road crossings. The kit contains six long-burning safety flares, a reflective ground sheet, and a latch-sealed case that keeps the contents dry. Each flare is individually wrapped and rated for outdoor use.',
        'explosives-weapons',
        12900,
        'USD'
    ),
    (
        'precision-signal-launcher',
        'Precision Signal Launcher',
        'A compact, manually operated signal launcher for trained outdoor crews.',
        'Designed for signalling across open terrain, the launcher has a machined grip, a positive safety catch, and a hard-shell carry case. It is supplied with a printed handling and storage guide. Intended for professional and trained recreational users only.',
        'explosives-weapons',
        34900,
        'USD'
    ),
    (
        'blast-resistant-storage-case',
        'Blast-Resistant Storage Case',
        'A reinforced field case for storing sensitive equipment and protective supplies.',
        'The double-wall shell, steel corner braces, and pressure-relief valve protect contents from impact, dust, and moisture. A removable tray keeps small parts organised, and the case stacks securely in a service vehicle or store room.',
        'explosives-weapons',
        28900,
        'USD'
    ),
    (
        'trailhead-navigation-unit',
        'Trailhead Navigation Unit',
        'A rugged handheld navigator with offline maps and a seven-day battery.',
        'Preloaded with regional trail maps and built to keep working away from a signal. The transflective display stays readable in direct sun, and physical buttons work with gloves. Includes a lanyard, USB-C cable, and a quick-start card.',
        'vehicles-gadgets',
        18900,
        'USD'
    ),
    (
        'compact-jump-starter',
        'Compact Jump Starter',
        'A pocket-sized lithium jump starter with reverse-polarity protection.',
        'Starts petrol engines up to 6 litres and diesel engines up to 3 litres, then recharges over USB-C. The insulated clamps and clear status lights make a roadside start straightforward. Stored in a padded zip case that fits under a seat.',
        'vehicles-gadgets',
        8900,
        'USD'
    ),
    (
        'all-weather-cargo-box',
        'All-Weather Cargo Box',
        'A lockable roof box with a low profile and a weather-tight seal.',
        'Adds 430 litres of covered space without turning the vehicle into a tall-load. The dual-side opening makes packing easy from either kerb, and the reinforced lid supports a roof bar accessory. Fits most factory and aftermarket crossbars.',
        'vehicles-gadgets',
        21900,
        'USD'
    )
ON CONFLICT (slug) DO NOTHING;
