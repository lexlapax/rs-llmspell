-- ABOUTME: Complex nested event data structures and serialization patterns
-- ABOUTME: Demonstrates handling of deep nested objects, arrays, mixed data types, and large payloads

print("=== Event Data Structures Example ===")
print("Demonstrates: Complex nested data, serialization, large payloads, and structured event handling")
print()

local subscriptions = {}
local data_stats = {
    events_published = 0,
    events_received = 0,
    total_data_size = 0,
    structure_types = {},
    nesting_levels = {}
}

-- Helper function to calculate data size (approximate)
local function estimate_data_size(data)
    local size = 0
    local function traverse(obj, depth)
        if type(obj) == "table" then
            for k, v in pairs(obj) do
                size = size + string.len(tostring(k)) + 8 -- key overhead
                traverse(v, depth + 1)
            end
        elseif type(obj) == "string" then
            size = size + string.len(obj)
        elseif type(obj) == "number" then
            size = size + 8
        elseif type(obj) == "boolean" then
            size = size + 1
        end
    end
    traverse(data, 0)
    return size
end

-- Helper function to analyze data structure
local function analyze_structure(data, max_depth)
    max_depth = max_depth or 0
    local structure_info = {
        depth = max_depth,
        object_count = 0,
        array_count = 0,
        primitive_count = 0,
        total_keys = 0
    }
    
    local function analyze(obj, current_depth)
        if type(obj) == "table" then
            local is_array = true
            local key_count = 0
            
            for k, v in pairs(obj) do
                key_count = key_count + 1
                if type(k) ~= "number" or k ~= key_count then
                    is_array = false
                end
                analyze(v, current_depth + 1)
            end
            
            structure_info.total_keys = structure_info.total_keys + key_count
            structure_info.depth = math.max(structure_info.depth, current_depth)
            
            if is_array and key_count > 0 then
                structure_info.array_count = structure_info.array_count + 1
            elseif key_count > 0 then
                structure_info.object_count = structure_info.object_count + 1
            end
        else
            structure_info.primitive_count = structure_info.primitive_count + 1
        end
    end
    
    analyze(data, 0)
    return structure_info
end

print("1. Setting up data structure subscriptions:")

local data_patterns = {
    simple_data = "data.simple.*",
    complex_data = "data.complex.*",
    nested_data = "data.nested.*",
    array_data = "data.array.*",
    mixed_data = "data.mixed.*",
    large_data = "data.large.*",
    structured_data = "data.structured.*",
    serialized_data = "data.serialized.*"
}

print("   📡 Creating data structure subscriptions:")
for pattern_name, pattern in pairs(data_patterns) do
    subscriptions[pattern_name] = Event.subscribe(pattern)
    print(string.format("   • %s: %s", pattern_name, pattern))
end

print("   ✅ Data structure event channels established")

print()
print("2. Simple data structures:")

print("   📤 Publishing simple data events:")

local simple_events = {
    {
        name = "data.simple.user_profile",
        data = {
            user_id = "user_12345",
            username = "john_doe",
            email = "john@example.com",
            age = 28,
            is_active = true,
            created_at = os.time(),
            last_login = os.time() - 3600
        }
    },
    {
        name = "data.simple.product",
        data = {
            product_id = "prod_abc123",
            name = "Wireless Headphones",
            price = 199.99,
            currency = "USD",
            in_stock = true,
            quantity = 50,
            category = "electronics",
            rating = 4.5
        }
    },
    {
        name = "data.simple.transaction",
        data = {
            transaction_id = "tx_789xyz",
            amount = 299.95,
            fee = 2.99,
            net_amount = 296.96,
            status = "completed",
            timestamp = os.time(),
            reference = "ORDER_2024_001"
        }
    }
}

for i, event in ipairs(simple_events) do
    local size = estimate_data_size(event.data)
    local structure = analyze_structure(event.data)
    
    local published = Event.publish(event.name, event.data)
    if published then
        data_stats.events_published = data_stats.events_published + 1
        data_stats.total_data_size = data_stats.total_data_size + size
        data_stats.structure_types["simple"] = (data_stats.structure_types["simple"] or 0) + 1
        data_stats.nesting_levels[structure.depth] = (data_stats.nesting_levels[structure.depth] or 0) + 1
        
        print(string.format("   %d. ✅ %s (~%d bytes, depth %d)", 
              i, event.name, size, structure.depth))
    end
end

print()
print("3. Complex nested data structures:")

print("   📤 Publishing complex nested events:")

local complex_events = {
    {
        name = "data.complex.user_session",
        data = {
            session_id = "sess_456def",
            user = {
                id = "user_12345",
                profile = {
                    personal = {
                        name = {
                            first = "John",
                            middle = "Robert",
                            last = "Doe",
                            display = "John R. Doe"
                        },
                        contact = {
                            email = {
                                primary = "john@example.com",
                                verified = true,
                                backup = "john.doe@gmail.com"
                            },
                            phone = {
                                primary = "+1-555-0123",
                                country_code = "US",
                                verified = false
                            },
                            address = {
                                street = "123 Main St",
                                apartment = "4B",
                                city = "New York",
                                state = "NY",
                                zip = "10001",
                                country = "USA",
                                coordinates = {
                                    latitude = 40.7128,
                                    longitude = -74.0060
                                }
                            }
                        }
                    },
                    preferences = {
                        theme = "dark",
                        language = "en-US",
                        timezone = "America/New_York",
                        notifications = {
                            email = true,
                            push = false,
                            sms = true,
                            frequency = "daily"
                        },
                        privacy = {
                            profile_visibility = "friends",
                            data_sharing = false,
                            analytics = true
                        }
                    }
                },
                permissions = {
                    roles = {"user", "beta_tester"},
                    capabilities = {
                        "read_profile",
                        "write_profile",
                        "access_beta_features"
                    },
                    restrictions = {
                        api_rate_limit = 1000,
                        daily_upload_mb = 100
                    }
                }
            },
            session = {
                start_time = os.time() - 7200,
                duration = 7200,
                activity = {
                    pages_visited = 15,
                    actions_performed = 47,
                    files_uploaded = 3,
                    api_calls_made = 128
                },
                device = {
                    type = "desktop",
                    os = "macOS",
                    browser = {
                        name = "Chrome",
                        version = "120.0.6099.109",
                        user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"
                    },
                    screen = {
                        width = 2560,
                        height = 1440,
                        color_depth = 24
                    }
                },
                location = {
                    ip_address = "203.0.113.45",
                    geolocation = {
                        country = "United States",
                        region = "New York",
                        city = "New York",
                        timezone = "America/New_York"
                    }
                }
            }
        }
    },
    {
        name = "data.complex.ecommerce_order",
        data = {
            order_id = "order_789ghi",
            customer = {
                id = "cust_456",
                tier = "premium",
                history = {
                    total_orders = 23,
                    total_spent = 2847.50,
                    average_order = 123.80,
                    first_order_date = os.time() - (365 * 24 * 3600),
                    last_order_date = os.time() - (30 * 24 * 3600)
                }
            },
            items = {
                {
                    product_id = "prod_abc123",
                    name = "Wireless Headphones",
                    sku = "WH-1000XM4",
                    quantity = 1,
                    price = {
                        unit = 199.99,
                        discount = 20.00,
                        final = 179.99,
                        currency = "USD"
                    },
                    attributes = {
                        color = "black",
                        warranty = "2_years",
                        gift_wrap = false
                    },
                    fulfillment = {
                        warehouse = "east_coast",
                        estimated_ship = os.time() + (24 * 3600),
                        tracking = {
                            carrier = "UPS",
                            service = "Ground",
                            estimated_delivery = os.time() + (5 * 24 * 3600)
                        }
                    }
                },
                {
                    product_id = "prod_def456",
                    name = "USB-C Cable",
                    sku = "USBC-3FT",
                    quantity = 2,
                    price = {
                        unit = 12.99,
                        discount = 0.00,
                        final = 12.99,
                        currency = "USD"
                    },
                    attributes = {
                        length = "3_feet",
                        color = "white",
                        fast_charging = true
                    }
                }
            },
            totals = {
                subtotal = 205.97,
                discount = 20.00,
                tax = 16.56,
                shipping = 5.99,
                total = 208.52,
                currency = "USD"
            },
            payment = {
                method = "credit_card",
                details = {
                    last_four = "1234",
                    brand = "visa",
                    expires = "12/27",
                    billing_address = {
                        same_as_shipping = true
                    }
                },
                transaction = {
                    id = "pay_xyz789",
                    status = "authorized",
                    gateway = "stripe",
                    fees = {
                        processing = 6.25,
                        currency_conversion = 0.00
                    }
                }
            },
            shipping = {
                address = {
                    recipient = "John Doe",
                    company = "",
                    street1 = "123 Main St",
                    street2 = "Apt 4B",
                    city = "New York",
                    state = "NY",
                    zip = "10001",
                    country = "US"
                },
                method = {
                    name = "Standard Delivery",
                    speed = "5-7 days",
                    cost = 5.99,
                    insurance = false
                }
            },
            metadata = {
                source = "web",
                campaign = "summer_sale_2024",
                referrer = "google_ads",
                user_agent = "Mozilla/5.0 (compatible)",
                notes = "Customer requested expedited processing"
            }
        }
    }
}

for i, event in ipairs(complex_events) do
    local size = estimate_data_size(event.data)
    local structure = analyze_structure(event.data)
    
    local published = Event.publish(event.name, event.data)
    if published then
        data_stats.events_published = data_stats.events_published + 1
        data_stats.total_data_size = data_stats.total_data_size + size
        data_stats.structure_types["complex"] = (data_stats.structure_types["complex"] or 0) + 1
        data_stats.nesting_levels[structure.depth] = (data_stats.nesting_levels[structure.depth] or 0) + 1
        
        print(string.format("   %d. ✅ %s (~%d bytes, depth %d, %d objects)", 
              i, event.name, size, structure.depth, structure.object_count))
    end
end

print()
print("4. Array-heavy data structures:")

print("   📤 Publishing array-heavy events:")

local array_events = {
    {
        name = "data.array.time_series",
        data = {
            metric_name = "cpu_usage",
            time_range = {
                start = os.time() - 3600,
                end = os.time(),
                interval = 60
            },
            data_points = (function()
                local points = {}
                for i = 1, 60 do -- 60 data points
                    table.insert(points, {
                        timestamp = os.time() - (3600 - (i * 60)),
                        value = math.random(10, 90) + math.random(),
                        metadata = {
                            source = "server_" .. math.random(1, 5),
                            quality = math.random() > 0.1 and "good" or "degraded"
                        }
                    })
                end
                return points
            end)(),
            aggregations = {
                min = 12.4,
                max = 87.9,
                avg = 45.2,
                median = 43.1,
                p95 = 78.5,
                p99 = 83.2
            },
            anomalies = {
                {
                    timestamp = os.time() - 1800,
                    value = 95.2,
                    type = "spike",
                    severity = "warning"
                },
                {
                    timestamp = os.time() - 900,
                    value = 2.1,
                    type = "drop",
                    severity = "critical"
                }
            }
        }
    },
    {
        name = "data.array.batch_processing",
        data = {
            batch_id = "batch_202401_001",
            processing_config = {
                chunk_size = 1000,
                parallel_workers = 8,
                retry_attempts = 3
            },
            items = (function()
                local items = {}
                for i = 1, 50 do -- 50 items
                    table.insert(items, {
                        id = "item_" .. string.format("%06d", i),
                        status = (function()
                            local statuses = {"pending", "processing", "completed", "failed"}
                            return statuses[math.random(1, #statuses)]
                        end)(),
                        size_bytes = math.random(1024, 1048576),
                        checksum = string.format("%032x", math.random(0, 2^32-1)),
                        metadata = {
                            source_file = "input_" .. math.ceil(i/10) .. ".dat",
                            content_type = "application/octet-stream",
                            encoding = "binary"
                        },
                        processing_history = (function()
                            local history = {}
                            local attempts = math.random(1, 3)
                            for j = 1, attempts do
                                table.insert(history, {
                                    attempt = j,
                                    timestamp = os.time() - (attempts * 60) + (j * 60),
                                    duration_ms = math.random(100, 5000),
                                    worker_id = "worker_" .. math.random(1, 8),
                                    result = j == attempts and "success" or "retry"
                                })
                            end
                            return history
                        end)()
                    })
                end
                return items
            end)(),
            statistics = {
                total_items = 50,
                completed = 42,
                failed = 3,
                pending = 5,
                total_size_bytes = 25678912,
                processing_time_seconds = 247
            }
        }
    }
}

for i, event in ipairs(array_events) do
    local size = estimate_data_size(event.data)
    local structure = analyze_structure(event.data)
    
    local published = Event.publish(event.name, event.data)
    if published then
        data_stats.events_published = data_stats.events_published + 1
        data_stats.total_data_size = data_stats.total_data_size + size
        data_stats.structure_types["array"] = (data_stats.structure_types["array"] or 0) + 1
        data_stats.nesting_levels[structure.depth] = (data_stats.nesting_levels[structure.depth] or 0) + 1
        
        print(string.format("   %d. ✅ %s (~%d bytes, %d arrays, %d total keys)", 
              i, event.name, size, structure.array_count, structure.total_keys))
    end
end

print()
print("5. Mixed data type structures:")

print("   📤 Publishing mixed data type events:")

local mixed_events = {
    {
        name = "data.mixed.scientific_data",
        data = {
            experiment_id = "exp_2024_001",
            researcher = {
                name = "Dr. Jane Smith",
                institution = "University Research Lab",
                contact = "jane.smith@university.edu"
            },
            parameters = {
                temperature_celsius = 23.5,
                humidity_percent = 65.2,
                pressure_hpa = 1013.25,
                ph_level = 7.4,
                is_controlled_environment = true,
                sample_size = 1000,
                duration_hours = 48.5
            },
            measurements = (function()
                local measurements = {}
                for i = 1, 20 do
                    table.insert(measurements, {
                        measurement_id = i,
                        timestamp = os.time() - (20 * 3600) + (i * 3600),
                        readings = {
                            primary = math.random() * 100,
                            secondary = math.random() * 50,
                            tertiary = math.random() * 25
                        },
                        quality_flags = {
                            calibrated = true,
                            within_range = math.random() > 0.05,
                            sensor_healthy = math.random() > 0.02
                        },
                        notes = i % 5 == 0 and "Calibration check performed" or nil
                    })
                end
                return measurements
            end)(),
            results = {
                statistical_summary = {
                    mean = 47.3,
                    median = 46.8,
                    mode = 45.2,
                    standard_deviation = 12.4,
                    variance = 153.76,
                    skewness = 0.23,
                    kurtosis = -0.67
                },
                correlation_matrix = {
                    {1.00, 0.73, -0.45},
                    {0.73, 1.00, -0.32},
                    {-0.45, -0.32, 1.00}
                },
                hypothesis_test = {
                    null_hypothesis = "No significant difference between groups",
                    p_value = 0.0234,
                    confidence_interval = {0.12, 0.89},
                    result = "reject_null",
                    significance_level = 0.05
                }
            },
            attachments = {
                raw_data_file = "experiment_001_raw.csv",
                analysis_script = "analysis.R",
                visualization = "results_plot.png",
                lab_notes = "experiment_001_notes.pdf"
            }
        }
    },
    {
        name = "data.mixed.multimedia_content",
        data = {
            content_id = "content_456789",
            type = "multimedia_article",
            metadata = {
                title = "Climate Change Impact on Polar Regions",
                author = {
                    name = "Environmental Research Team",
                    credentials = {"PhD Environmental Science", "Climate Research Specialist"},
                    affiliations = {"Arctic Research Institute", "Climate Change Coalition"}
                },
                publication = {
                    date = os.time() - (7 * 24 * 3600),
                    journal = "Environmental Science Today",
                    doi = "10.1234/est.2024.001",
                    peer_reviewed = true
                },
                topics = {"climate change", "polar regions", "environmental impact", "research"},
                reading_level = "graduate",
                estimated_read_time_minutes = 15
            },
            content_structure = {
                sections = {
                    {
                        title = "Abstract",
                        word_count = 250,
                        has_images = false,
                        has_charts = false
                    },
                    {
                        title = "Introduction",
                        word_count = 800,
                        has_images = true,
                        image_count = 2,
                        has_charts = false
                    },
                    {
                        title = "Methodology",
                        word_count = 600,
                        has_images = false,
                        has_charts = true,
                        chart_count = 3
                    },
                    {
                        title = "Results",
                        word_count = 1200,
                        has_images = true,
                        image_count = 4,
                        has_charts = true,
                        chart_count = 6
                    },
                    {
                        title = "Discussion",
                        word_count = 900,
                        has_images = false,
                        has_charts = false
                    },
                    {
                        title = "Conclusion",
                        word_count = 300,
                        has_images = false,
                        has_charts = false
                    }
                },
                media_assets = {
                    images = (function()
                        local images = {}
                        for i = 1, 6 do
                            table.insert(images, {
                                filename = "figure_" .. i .. ".jpg",
                                caption = "Figure " .. i .. ": Research findings visualization",
                                resolution = {width = 1920, height = 1080},
                                file_size_bytes = math.random(500000, 2000000),
                                format = "jpeg",
                                color_space = "sRGB"
                            })
                        end
                        return images
                    end)(),
                    charts = (function()
                        local charts = {}
                        for i = 1, 9 do
                            table.insert(charts, {
                                filename = "chart_" .. i .. ".svg",
                                type = ({"bar", "line", "scatter", "pie"})[math.random(1, 4)],
                                data_points = math.random(10, 100),
                                interactive = math.random() > 0.5,
                                file_size_bytes = math.random(50000, 200000)
                            })
                        end
                        return charts
                    end)(),
                    videos = {
                        {
                            filename = "polar_timelapse.mp4",
                            duration_seconds = 120,
                            resolution = {width = 3840, height = 2160},
                            fps = 30,
                            codec = "h264",
                            file_size_bytes = 45000000,
                            has_subtitles = true,
                            language = "en"
                        }
                    }
                },
                references = (function()
                    local refs = {}
                    for i = 1, 25 do
                        table.insert(refs, {
                            id = i,
                            type = ({"journal", "book", "conference", "website"})[math.random(1, 4)],
                            authors = {"Author " .. i .. "A", "Author " .. i .. "B"},
                            title = "Research Paper Title " .. i,
                            year = 2020 + math.random(0, 4),
                            cited_count = math.random(1, 5)
                        })
                    end
                    return refs
                end)()
            },
            engagement_metrics = {
                views = 1247,
                downloads = 89,
                citations = 3,
                social_shares = {
                    twitter = 23,
                    linkedin = 15,
                    facebook = 8,
                    reddit = 41
                },
                reader_feedback = {
                    average_rating = 4.3,
                    total_ratings = 67,
                    comments_count = 12,
                    bookmark_count = 156
                }
            }
        }
    }
}

for i, event in ipairs(mixed_events) do
    local size = estimate_data_size(event.data)
    local structure = analyze_structure(event.data)
    
    local published = Event.publish(event.name, event.data)
    if published then
        data_stats.events_published = data_stats.events_published + 1
        data_stats.total_data_size = data_stats.total_data_size + size
        data_stats.structure_types["mixed"] = (data_stats.structure_types["mixed"] or 0) + 1
        data_stats.nesting_levels[structure.depth] = (data_stats.nesting_levels[structure.depth] or 0) + 1
        
        print(string.format("   %d. ✅ %s (~%d bytes, depth %d, %d primitives)", 
              i, event.name, size, structure.depth, structure.primitive_count))
    end
end

print()
print("6. Large payload data structures:")

print("   📤 Publishing large payload events:")

-- Generate a large data structure
local large_data = {
    dataset_id = "large_dataset_001",
    metadata = {
        created_at = os.time(),
        size_classification = "large",
        estimated_memory_mb = 5.2
    },
    data_matrix = {}
}

-- Create a large nested structure
for i = 1, 50 do -- 50 rows
    large_data.data_matrix[i] = {}
    for j = 1, 20 do -- 20 columns per row
        large_data.data_matrix[i][j] = {
            row = i,
            col = j,
            value = math.random() * 1000,
            computed_fields = {
                normalized = math.random(),
                squared = math.random() * 1000000,
                logarithmic = math.log(math.random() * 100 + 1),
                trigonometric = {
                    sin = math.sin(math.random() * math.pi),
                    cos = math.cos(math.random() * math.pi),
                    tan = math.tan(math.random() * math.pi/4)
                }
            },
            metadata = {
                last_updated = os.time() - math.random(86400),
                data_source = "sensor_" .. math.random(1, 10),
                quality_score = math.random(),
                flags = {
                    validated = math.random() > 0.1,
                    outlier = math.random() < 0.05,
                    interpolated = math.random() < 0.02
                }
            }
        }
    end
end

local large_event = {
    name = "data.large.matrix_dataset",
    data = large_data
}

local size = estimate_data_size(large_event.data)
local structure = analyze_structure(large_event.data)

local published = Event.publish(large_event.name, large_event.data)
if published then
    data_stats.events_published = data_stats.events_published + 1
    data_stats.total_data_size = data_stats.total_data_size + size
    data_stats.structure_types["large"] = (data_stats.structure_types["large"] or 0) + 1
    data_stats.nesting_levels[structure.depth] = (data_stats.nesting_levels[structure.depth] or 0) + 1
    
    print(string.format("   1. ✅ %s (~%.1f KB, depth %d, %d total objects)", 
          large_event.name, size/1024, structure.depth, structure.object_count))
end

print()
print("7. Receiving and analyzing structured data:")

print("   📥 Processing structured data events:")

-- Process each subscription type
local processing_results = {}

for subscription_name, sub_id in pairs(subscriptions) do
    local events_received = {}
    
    -- Try to receive multiple events
    for attempt = 1, 3 do
        local received = Event.receive(sub_id, 300) -- 300ms timeout
        if received then
            table.insert(events_received, received)
            data_stats.events_received = data_stats.events_received + 1
        else
            break
        end
    end
    
    if #events_received > 0 then
        print(string.format("   📨 %s: %d events received", subscription_name, #events_received))
        
        for i, event in ipairs(events_received) do
            if event.data then
                local size = estimate_data_size(event.data)
                local structure = analyze_structure(event.data)
                
                print(string.format("     %d. %s: ~%d bytes, depth %d", 
                      i, event.event_type or "unknown", size, structure.depth))
                
                -- Analyze specific data patterns
                if event.data.user then
                    print("       • Contains user data structure")
                end
                if event.data.items and type(event.data.items) == "table" then
                    print(string.format("       • Contains %d items array", #event.data.items))
                end
                if event.data.data_points and type(event.data.data_points) == "table" then
                    print(string.format("       • Contains %d data points", #event.data.data_points))
                end
                if event.data.data_matrix and type(event.data.data_matrix) == "table" then
                    print(string.format("       • Contains %dx%d data matrix", 
                          #event.data.data_matrix, 
                          event.data.data_matrix[1] and #event.data.data_matrix[1] or 0))
                end
            end
        end
        
        processing_results[subscription_name] = events_received
    else
        print(string.format("   ⏰ %s: no events received", subscription_name))
    end
end

print()
print("8. Data structure analysis:")

print("   📊 Data Structure Statistics:")
print("   • Total events published:", data_stats.events_published)
print("   • Total events received:", data_stats.events_received)
print(string.format("   • Total data transferred: ~%.1f KB", data_stats.total_data_size / 1024))
print(string.format("   • Average event size: ~%.1f bytes", 
      data_stats.events_published > 0 and (data_stats.total_data_size / data_stats.events_published) or 0))

print()
print("   🏷️  Structure Type Distribution:")
for structure_type, count in pairs(data_stats.structure_types) do
    print(string.format("   • %s structures: %d events", structure_type, count))
end

print()
print("   📏 Nesting Depth Distribution:")
for depth, count in pairs(data_stats.nesting_levels) do
    print(string.format("   • Depth %d: %d events", depth, count))
end

print()
print("9. Data serialization best practices:")

print("   💡 Data Structure Best Practices:")
print("   • Use consistent field naming conventions across events")
print("   • Include metadata for data provenance and quality")
print("   • Structure arrays for efficient processing and iteration")
print("   • Implement data validation at event boundaries")
print("   • Consider memory usage for large nested structures")
print("   • Use appropriate data types for performance optimization")
print("   • Include versioning information for schema evolution")
print("   • Implement compression for large payload events")

print()
print("10. Performance considerations:")

print("   ⚡ Performance Analysis:")

-- Measure serialization/deserialization performance
local perf_start = os.clock()
for i = 1, 10 do
    local test_data = {
        id = "perf_test_" .. i,
        nested = {
            level1 = {
                level2 = {
                    level3 = {
                        data = string.rep("x", 1000) -- 1KB string
                    }
                }
            }
        },
        array = (function()
            local arr = {}
            for j = 1, 100 do
                arr[j] = {id = j, value = math.random()}
            end
            return arr
        end)()
    }
    
    Event.publish("data.performance.test", test_data)
end
local perf_time = (os.clock() - perf_start) * 1000

print(string.format("   • Large event publishing: %.2fms (10 events)", perf_time))
print("   • Estimated throughput: ~" .. math.floor(10 / (perf_time / 1000)) .. " events/second")

print()
print("11. Cleaning up data structure subscriptions:")

local cleanup_count = 0
for name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        cleanup_count = cleanup_count + 1
        print("   🧹 Unsubscribed from", name)
    end
end

local final_subs = Event.list_subscriptions()
print("   ✅ Cleaned up", cleanup_count, "data structure subscriptions")
print("   ✅ Final subscription count:", #final_subs)

print()
print("✨ Event data structures example complete!")
print("   Key concepts demonstrated:")
print("   • Simple flat data structures with primitive types")
print("   • Complex deeply nested object hierarchies")
print("   • Array-heavy data structures with time series and batch data")
print("   • Mixed data types combining numbers, strings, booleans, and objects")
print("   • Large payload handling with performance considerations")
print("   • Data structure analysis and metrics collection")
print("   • Serialization performance and optimization techniques")
print("   • Best practices for structured event data design")