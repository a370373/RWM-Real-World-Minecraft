## 🗺️ RWM — Real World Minecraft

«把真實世界帶進 Minecraft，不只是複製地圖，而是重建它的空間。»

# RWM（Real World Minecraft）是一個 Local-First、CLI-First、Multi-Source Real-World Reconstruction Engine。

RWM 將真實世界的：

- 🌍 地理位置
- 🗺️ OpenStreetMap 資料
- 🏢 Overture Maps 建築資料
- 🛣️ 道路與交通結構
- ⛰️ 真實高程
- 🌱 土地覆蓋
- 🌦️ 氣候與環境資料
- 🏛️ 3D Models / Landmarks
- 🪟 建築外觀與既有窗戶
- 🏠 建築空間資訊

經過資料處理、地理映射、資料融合、世界理解與程序化生成後，建立可以直接在 Minecraft 中使用的真實世界。

RWM 的核心目標不是：

«Real Map → Minecraft»

而是：

«Real World → Data → Understanding → Reconstruction → Minecraft»

---

## ✨ Core Philosophy

RWM 的設計建立在四個核心原則：

# 🌍 Real-World First

真實世界資料是生成的基礎，而不是單純使用隨機程序生成一個「看起來像城市」的世界。

RWM 的目標是盡可能從真實世界資料理解：

- 地理位置
- 空間關係
- 地形
- 建築
- 道路
- 基礎設施
- 環境
- 建築用途
- 建築空間

再將這些資訊重建為 Minecraft 世界。

## 💾 Local-First

RWM 優先使用本地已有的：

- Bundled Assets
- 本地模型
- 本地 NBTX / Minecraft Assets
- Local Cache
- 本地資料

只有在本地沒有可用資料時，才向公開第三方資料來源取得資料。

Bundled Assets
↓
Local Assets / Cache
↓
Public Data Sources

網路是資料來源，不是 RWM 核心 Engine 的依賴。

對於已經取得並存在本地的資料，RWM 可以在沒有再次連線資料來源的情況下進行後續世界生成。

## ⚙️ CLI-First

RWM 的核心 Engine 不依賴 GUI。

所有核心生成能力都可以由 CLI 驅動。

GUI 若存在，只是 RWM 的使用者介面層，而不是核心生成系統。

核心關係：

CLI
↓
RWM Engine
↓
World Generation

而不是：

GUI
↓
GUI-specific Logic
↓
World Generation

## 🧩 Multi-Source

RWM 不依賴單一資料庫或單一官方服務。

不同資料來源負責不同種類的真實世界資訊，再由 RWM Data Processing / Fusion Layer 統一處理。

因此資料來源可以被替換、擴充或由其他來源補足。

---

## 🌍 1. Real-World Geographic Selection

RWM 從真實地球上的 Geographic Bounding Box（BBox）開始。

使用者可以指定：

min latitude
min longitude
max latitude
max longitude

BBox 可以代表：

- 一棟建築
- 一個街區
- 一個社區
- 一座校園
- 一個城市區域
- 一段道路
- 山區
- 海岸
- 城市
- 任意地理區域

BBox 是 RWM 世界重建的 authoritative geographic boundary。

BBox 內的真實世界資料是 World Reconstruction 的核心範圍。

---

## 📐 2. Geographic → Minecraft Mapping

RWM 將：

Real-World Coordinates
↓
Geographic Projection
↓
Minecraft Coordinates

支援：

- Local Projection
- Web Mercator Projection
- Adjustable Scale
- Real-world distance mapping

預設 Scale：

1.0

代表約：

1 real-world meter ≈ 1 Minecraft block

這是地理空間與尺度的 1:1 映射。

實際還原精度仍取決於：

- 公開資料解析度
- 資料完整度
- 資料來源
- RWM processing
- Minecraft block representation

因此「1:1」代表空間尺度與地理位置映射，而不是聲稱每一個現實世界物件都能以相同精度還原到每一個 Minecraft Block。

---

## 🌐 3. Multi-Source Real-World Data

RWM 使用多種公開與第三方資料來源。

資料會經過：

Data Acquisition
↓
Data Processing
↓
Data Normalization
↓
Data Fusion
↓
World Understanding
↓
World Generation

主要資料來源包括：

Data | Source
Geographic / Map Data | OpenStreetMap
OSM Query | Overpass API
Reverse Geocoding | Nominatim
Building Data | Overture Maps
Building STAC / S3 | Overture Maps
Elevation | Mapterhorn
Elevation | AWS Terrain Tiles
Regional Elevation | USGS 3DEP
Land Cover | ESA WorldCover
3D Models | 3DMR
3D Models / Media | Wikimedia Commons
3D / Metadata | Wikidata
Local Minecraft Assets | RWM Bundled / Local Assets

RWM 不依賴單一中央世界資料庫。

---

## 🗺️ 4. OpenStreetMap Integration

OpenStreetMap（OSM）提供主要的真實世界地理與人造環境資訊。

可以處理：

- Roads
- Buildings
- Waterways
- Railways
- Bridges
- Parks
- Public Facilities
- Amenities
- Barriers
- Power Infrastructure
- Transportation
- Street Objects
- Geographic Objects
- Building Tags
- Building Metadata

OSM 資料透過公開 Overpass 服務取得。

RWM 也使用 Nominatim 進行 Reverse Geocoding。

RWM 使用多個公開 Overpass endpoint，以降低對單一服務端點的依賴。

---

## 🏢 5. Overture Maps Integration

Overture Maps 用於補充與增強建築及其他地理資料。

特別用於：

- Building Footprints
- Building Geometry
- Building Location
- Building Attributes
- 建築資料補充
- OSM 缺失的建築資料

RWM 可以將 OSM 與 Overture 的建築資訊進行資料融合。

OSM
+
Overture
↓
Building Dataset
↓
RWM Building Engine

Overture Maps 的公開 STAC / S3 資料來源可作為建築資料取得管線的一部分。

---

## ⛰️ 6. Elevation System

RWM 使用真實世界 Elevation Data 建立 Minecraft Terrain。

支援多來源 Elevation：

- Mapterhorn
- AWS Terrain Tiles
- USGS 3DEP
- Regional Elevation Providers

資料流程：

Elevation Data
↓
Elevation Processing
↓
Terrain Reconstruction
↓
Minecraft Terrain

可以重建：

- Mountains
- Hills
- Valleys
- Plains
- Slopes
- Elevation Differences
- Coastlines
- Terrain Features

不同 Elevation Provider 可以作為替代或補充來源。

---

## 🌱 7. Land Cover System

RWM 可以使用 ESA WorldCover 等土地覆蓋資料判斷真實世界地表。

包括：

- Forest
- Grassland
- Cropland
- Built-up
- Water
- Bare / Sparse Vegetation
- Snow / Ice

土地覆蓋可以參與：

Land Cover
↓
Environment Understanding
↓
Terrain / Vegetation Generation

使自然環境具有真實世界資料基礎。

---

## 🌦️ 8. Climate / Environmental Data

RWM 的環境系統可以使用氣候與環境分類資料作為：

- Climate Classification
- Environment Analysis
- Vegetation Decisions
- Regional Environment
- Future Biome Systems

的資料基礎。

氣候資料與土地覆蓋、Elevation、水域等資料可以共同參與環境理解。

此層目前主要作為 RWM Environment / World Understanding 的擴充方向；具體資料來源與生成規則會依實際實作持續擴充。

---

## 🏛️ 9. 3D Model & Landmark System

RWM 支援第三方 3D Model / Landmark Data。

目前整合方向包括：

- 3DMR
- Wikimedia Commons
- Wikidata

可以用於：

- Buildings / Landmarks
- City Objects
- Vehicles
- Structures
- Real-world 3D Models
- Model Metadata
- Licensing Metadata

RWM 同時支援 Bundled / Local Assets，降低對網路模型來源的依賴。

第三方模型來源與 RWM 自有 / Bundled Assets 在架構上保持分離。

---

## 💾 10. Local-First Asset System

RWM 的資產解析遵循：

RWM Bundled Assets
↓
Local Assets / Cache
↓
Third-Party Network Sources

如果本地已經存在可使用的：

- Model
- NBTX Asset
- Schematic
- Tree
- Vehicle
- Structure
- Other RWM Asset

RWM 不需要重新下載。

這讓 RWM 能夠逐漸建立自己的本地資產庫。

Local-First 不代表所有生成工作都必須永久離線，而是：

「已存在本地的資料與資產應優先被使用，而不是無條件重新依賴網路來源。」

---

## 🧱 11. RWM Asset / NBTX System

RWM 使用本地與 Bundled Minecraft assets 建立世界中的物件。

包括：

- Trees
- Cars
- Boats
- Cranes
- Excavators
- Helicopters
- Fountains
- Playgrounds
- Lighthouses
- Tractors
- Wind Turbines
- Tombstones
- Starships
- Props
- Minecraft Schematics
- NBTX-based structures
- Bundled Models

這些資產屬於 RWM World Generation Pipeline 的物件層。

NBTX / Minecraft assets 可以作為已取得的本地或 Bundled 資產被 World Generation Layer 使用。

---

## 🧠 12. RWM World Reconstruction Engine

RWM World Engine 是整個系統的核心。

Real-World Data
↓
Data Processing
↓
Geographic Reconstruction
↓
Data Fusion
↓
World Understanding
↓
Procedural World Generation
↓
Minecraft World

World Engine 負責：

- 1. Data Parsing
- 2. Geographic Processing
- 3. Coordinate Mapping
- 4. Multi-Source Data Fusion
- 5. Terrain Reconstruction
- 6. Building Reconstruction
- 7. Road Reconstruction
- 8. Infrastructure Generation
- 9. Environment Generation
- 10. Object Generation
- 11. World Output

# World Reconstruction Engine 對 BBox 內的真實世界幾何具有 authoritative role。

---

## 🏙️ 13. Real-World Structure Engine

RWM 可以將真實世界的人造環境重建成 Minecraft 結構。

包括：

- Buildings
- Roads
- Bridges
- Railways
- Waterways
- Parks
- Facilities
- Amenities
- Infrastructure
- Street Objects
- Other Geographic Objects

例如：

Real Road Data
↓
Road Processing
↓
Road Generation
↓
Minecraft Road

以及：

Building Footprint
↓
Building Reconstruction
↓
Minecraft Building

---

## 🏢 14. Building Reconstruction Engine

RWM 的世界還原核心負責建立真實世界建築的：

- Footprint
- Geometry
- Height
- Floors
- Roof
- Walls
- Building Materials
- Exterior Structure
- Windows
- Glass
- Exterior Features

這個系統是 World Reconstruction Layer。

它負責：

«「真實世界中的這棟建築長什麼樣？」»

# World Reconstruction Engine 是建築 Exterior Geometry 的 authoritative source。

---

## 🧠 15. Building Intelligence Engine

RWM 在世界還原 Engine 之外，加入獨立的 Building Intelligence Layer。

它負責：

«「這棟建築應該怎麼被理解？」»

Building Intelligence 可以讀取：

- OSM Metadata
- Overture Metadata
- Building Type
- Building Geometry
- Footprint
- Width
- Depth
- Floors
- Existing Entrances
- Existing Windows
- Window Position
- Window Dimensions
- Building Environment
- Daylight Information

然後推導：

- Building Type
- Spatial Requirements
- Room Requirements
- Floor Plan
- Entrance Assignment
- Room Connectivity
- Furniture Requirements
- Lighting Requirements
- Interior Circulation

# Building Intelligence 是 interpretation / planning layer，而不是 Exterior Reconstruction Layer。

---

## 🪟 16. Existing Building Geometry & Window Intelligence

RWM Interior Engine 不重新生成建築外窗。

這一點是 RWM 建築架構的重要設計原則。

世界還原 Engine 已經建立：

Building
├── Exterior Geometry
├── Walls
├── Floors
├── Roof
└── Existing Windows

Interior Intelligence 只讀取這些資訊。

例如：

Existing Windows
↓
Window Position
Window Width
Window Height
Window Side
Window Floor
↓
Daylight Analysis
↓
Room Planning

因此室內系統知道：

«哪些位置已經有窗戶、哪一面有採光、哪些空間適合自然採光。»

Interior Engine 不負責：

- 移動外窗
- 刪除外窗
- 重新生成外窗
- 修改建築 Footprint
- 修改建築 Exterior Geometry
- 修改 BBox

---

## ☀️ 17. Daylight Intelligence

RWM 可以分析既有建築窗戶提供的自然採光資訊。

考慮：

- Window Width
- Window Height
- Window Area
- Window Side
- Window Floor
- Existing Window Distribution

然後建立 Daylight Information。

Daylight 可以影響：

- Room Allocation
- Naturally Lit Rooms
- Room Placement
- Interior Lighting Decisions

因此窗戶不是單純的裝飾，而是：

Real-World Window
↓
Building Intelligence
↓
Daylight Analysis
↓
Interior Planning

---

## 🏠 18. Procedural Interior Engine

RWM 的 Interior Engine 不只是：

«「在建築裡塞家具。」»

而是建築空間生成系統。

流程：

Building Type
↓
Building Geometry
↓
Floor Information
↓
Existing Windows
↓
Daylight
↓
Entrance / Environment
↓
Room Requirements
↓
Room Allocation
↓
Floor Plan
↓
Room Graph
↓
Doors
↓
Circulation
↓
Furniture
↓
Lighting
↓
Interior Decoration

Interior Engine 的輸入是既有 World Reconstruction 結果。

它不重新定義建築本身。

---

## 🏢 19. Building Type System

RWM 可以根據 OSM / Overture / Building Intelligence 判斷建築用途。

目前包含多種 Building Types，例如：

- Residential
- Restaurant
- Shop
- Supermarket
- Mall
- Office
- Corporate
- Government
- School
- College
- University
- Hospital

不同建築類型具有不同的空間需求。

Building Type 可以影響：

- Room Requirements
- Room Allocation
- Floor Planning
- Furniture Profiles
- Lighting Decisions
- Interior Layout

---

## 🏫 20. Building-Type Room Templates

例如 School：

School
├── Entrance
├── Lobby
├── Corridor
├── Classroom × N
├── Teacher / Office Space
├── Toilet
└── Storage

Hospital：

Hospital
├── Main Entrance
├── Reception
├── Corridor
├── Examination Room × N
├── Ward × N
├── Nursing Station
├── Toilet
└── Utility / Storage

不同 Building Type 使用不同 Room Requirements。

# Room templates are planning rules rather than a guarantee that every real-world building contains exactly the same rooms.

---

## 📐 21. Intelligent Room Allocation

Room Allocation 不只是固定數量。

系統可以根據：

- Building Type
- Building Area
- Building Width
- Building Depth
- Floors
- Available Space
- Room Requirements
- Daylight
- Spatial Constraints

決定：

- 房間數量
- 房間類型
- 房間大小
- Minimum Dimensions
- Preferred Floor
- Daylight Requirements

因此：

Small Building
↓
Smaller Room Plan

Large Building
↓
More Rooms
↓
More Complex Layout

---

## 🧩 22. Procedural Floor Plan

Floor Plan Engine 根據實際建築幾何生成空間配置。

考慮：

- Building Footprint
- Building Bounds
- Building Shape
- Building Width / Depth
- Floors
- Room Requirements
- Daylight
- Spatial Constraints

每棟建築不需要使用完全相同的模板。

同一種 Building Type 可以產生不同的 Floor Plan。

# Floor Plan 必須受既有建築幾何約束，而不是重新定義 Exterior Geometry。

---

## 🕸️ 23. Room Graph / Spatial Topology

RWM 將室內空間視為 Spatial Graph。

例如：

Entrance
↓
Lobby
↓
Corridor
┌─┴────────┐
↓          ↓
Room A     Room B
↓          ↓
Storage    Toilet

Room Graph 用於：

- Room Connectivity
- Door Placement
- Circulation
- Accessibility
- Interior Navigation
- Furniture Clearance

核心問題是：

«「人是否真的能從入口走到建築裡的重要空間？」»

而不是只把房間畫出來。

---

## 🚪 24. Entrance & Door Intelligence

RWM 處理兩種入口：

Exterior Entrance

Outside
↓
Building Entrance
↓
Interior

系統會在可取得真實入口資訊時使用相關資訊。

如果資料不足，可以根據：

- Building Geometry
- Building Boundary
- Existing Entrance Data
- Road Relationship
- Exterior Environment
- Building Orientation
- Spatial Availability

推導合理入口候選位置。

Interior Doors

Room A
↓
Interior Door
↓
Room B

門的位置考慮：

- Shared Walls
- Room Connectivity
- Room Geometry
- Furniture
- Circulation
- Building Structure

# 門因此是空間拓撲的一部分，而不是隨機放置的裝飾。

---

## 🚶 25. Interior Circulation

RWM Interior Engine 具有室內動線規劃。

系統會考慮：

- Corridors
- Doorways
- Furniture Obstacles
- Room Connectivity
- Walkable Space
- Player Clearance
- Pathfinding
- Furniture Clearance

目標是：

«生成一個「可以走」的室內，而不只是看起來像房間。»

---

## 🪑 26. Room-Aware Furniture System

家具由：

Building
↓
Room
↓
Room Type
↓
Furniture Profile
↓
Placement

決定。

不同 Room Type 使用不同 Furniture Profile。

例如：

Classroom

- Classroom Desk
- Chair

Ward

- Hospital Bed
- Medical Desk

Examination Room

- Hospital Bed
- Medical Desk

Nursing Station

- Desk
- Chair

Storage

- Storage Shelf

Furniture Placement 同時考慮：

- Room Bounds
- Walls
- Doors
- Walkways
- Existing Objects
- Clearance

避免家具阻塞主要通行空間。

---

## 💡 27. Interior Lighting

Interior Engine 可以根據：

- Room Type
- Room Geometry
- Daylight
- Existing Windows
- Room Usage
- Interior Layout

建立室內人工照明配置。

# 自然採光與人工照明因此是同一個 Interior Planning Pipeline 的一部分。

---

## 🎲 28. Procedural Variation

RWM 不希望所有建築都完全一樣。

生成結果可以受到：

- Building Size
- Building Shape
- Floors
- Room Requirements
- Available Space
- Random Seed
- Building Type

影響。

因此即使兩棟建築都是同一種類型，也可以產生不同的：

- Room Layout
- Room Count
- Door Layout
- Furniture Placement
- Interior Arrangement

---

## 🏠 29. Interior Engine Safety Boundary

RWM 明確區分：

WORLD RECONSTRUCTION ENGINE
│
│ authoritative
▼
Existing Building
├── Geometry
├── Footprint
├── Floors
├── Exterior
└── Existing Windows
│
│ READ-ONLY
▼
BUILDING INTELLIGENCE
│
▼
INTERIOR ENGINE

Interior Engine 可以：

- Read Building Geometry
- Read Existing Windows
- Read Building Type
- Read Floors
- Read Entrance Information
- Analyze Daylight
- Plan Rooms
- Generate Interior Doors
- Generate Interior Walls
- Generate Furniture
- Generate Lighting
- Generate Interior Circulation

Interior Engine 不應：

- 修改 BBox
- 修改 Geographic Coordinates
- 修改 Building Footprint
- 移動 Exterior Windows
- 刪除 Exterior Windows
- 修改 Exterior Reconstruction
- 修改原始世界還原資料

# World Reconstruction remains authoritative.

---

## 🌳 30. Terrain & Environment Generation

RWM 將：

Elevation
+
Land Cover
+
Climate
+
Water

融合成 Environment Generation。

可生成：

- Terrain
- Trees
- Vegetation
- Grass
- Natural Ground
- Water Environment
- Regional Environmental Features

---

# 🌊 31. Outside Terrain

BBox 外的世界可以由使用者指定。

例如：

- Ocean
- Superflat
- Normal
- Void
- Desert
- Snow

核心規則：

BBox Inside
↓
Real-World Reconstruction

BBox Outside
↓
User-Selected Outside Terrain

Outside Terrain 不應覆蓋 BBox 內的真實世界區域。

# Outside Terrain 是 BBox 外部環境生成層，而不是 BBox 內真實世界重建的一部分。

---

## 📏 32. Outside Padding

使用者可以控制 BBox 周圍額外生成的區域。

可以建立：

Real-World Area
+
Controlled Outside Environment

例如：

Real Taiwan Region + Ocean

或：

Real City + Superflat

或：

Real Mountain Area + Void

Outside Padding 與 BBox 是獨立的空間控制機制。

BBox 定義真實世界重建範圍。

Outside Padding 定義 BBox 外可額外延伸的生成空間。

---

## 🎮 33. Minecraft World Output

RWM 可以將生成結果輸出為 Minecraft 世界。

支援方向包括：

Java Edition

Minecraft Anvil World：

region/r.x.z.mca

Bedrock Edition

.mcworld

Luanti / Minetest

map.sqlite

World Output Layer 將 RWM World Engine 的結果寫入對應世界格式。

---

## ⚙️ 34. CLI-First

RWM 核心 Engine 以 CLI 為主要控制介面。

CLI 可以控制的核心設定包括：

- BBox
- Output Directory
- Scale
- Projection
- Ground Level
- Generation Mode
- Outside Terrain
- Outside Padding
- Overture
- Interior Generation
- 3D Models
- Game Mode
- World Output
- 其他生成選項

核心理念：

CLI
↓
RWM Engine
↓
World Generation

而不是：

GUI
↓
GUI-specific Logic
↓
World Generation

RWM 的核心能力不應依賴 GUI。

---

## 💾 35. Local-First

RWM 的資料與資產策略：

LOCAL FIRST
│
├── Bundled Assets
├── Local NBTX
├── Local Models
├── Local Cache
└── Local Data
│
▼
Network Sources

這代表：

- 已有本地資產不重複下載
- 本地模型可以持續累積
- 本地 NBTX 可以直接使用
- 公開資料來源可以替換
- 不依賴單一中央服務
- RWM 核心生成能力可以對已取得的本地資料進行後續處理

# Local-First 是資料與資產策略，不等於所有 Real-World Data Acquisition 都必須永久離線。

---

## 🧩 36. Resilient Multi-Source Architecture

RWM 不把所有能力綁定在單一 API。

例如 Elevation 可以使用不同來源：

Elevation
├── Mapterhorn
├── AWS Terrain
├── USGS 3DEP
└── Regional Providers

建築：

Building Data
├── OSM
└── Overture

3D：

3D Models
├── Local Assets
├── 3DMR
├── Wikimedia
└── Wikidata

因此某個第三方來源失效時，RWM 架構仍可以使用其他來源或本地資料。

# 這也是 RWM Multi-Source / Resilient Architecture 的核心設計之一。

---

## 🏗️ 37. Complete RWM Architecture

                     🌍 REAL WORLD
                          │
                          ▼
                   📍 GEOGRAPHIC BBOX
                          │
                          ▼
                 📐 SPATIAL MAPPING
                          │
                          ▼
                🌐 DATA ACQUISITION
                          │
    ┌─────────────────────┼─────────────────────┐
    │          │          │          │           │
   OSM      Overture  Elevation  Land Cover  3D Data
    │          │          │          │           │
    └──────────┴──────────┴──────────┴───────────┘
                          │
                          ▼
                 🧠 DATA PROCESSING
                          │
                          ▼
                   🔀 DATA FUSION
                          │
                          ▼
              🧠 RWM WORLD ENGINE
                          │
   ┌──────────────────────┼──────────────────────┐
   │                      │                      │
   ▼                      ▼                      ▼

🏙️ STRUCTURES           ⛰️ TERRAIN           🌳 ENVIRONMENT
│                      │                      │
▼                      ▼                      ▼
🏢 BUILDING ENGINE      Elevation            Land Cover
│                Processing            Climate
▼                                      Water
🧠 BUILDING INTELLIGENCE
│
├── Building Type
├── Building Geometry
├── Existing Windows
├── Daylight
├── Entrance Analysis
├── Room Requirements
├── Room Allocation
├── Floor Plan
├── Room Graph
├── Doors
├── Circulation
├── Furniture
└── Lighting
│
▼
🏠 PROCEDURAL INTERIOR ENGINE
│
▼
🧱 3D OBJECT / ASSET LAYER
│
├── RWM Bundled Assets
├── Local Assets / NBTX
└── Third-Party Models
│
▼
🌊 OUTSIDE TERRAIN
│
▼
🎮 WORLD OUTPUT
│
├── Java
├── Bedrock
└── Luanti
│
▼
🌍 RWM WORLD

---

## 🧭 38. End-to-End Pipeline

完整生成流程：

Real World
↓
Geographic BBox
↓
Data Acquisition
↓
OSM / Overture / Elevation / Land Cover / 3D
↓
Data Processing
↓
Geographic Reconstruction
↓
World Understanding
↓
World Reconstruction
├── Terrain
├── Roads
├── Buildings
├── Infrastructure
├── Water
├── Vegetation
└── Objects
↓
Building Intelligence
├── Building Type
├── Geometry
├── Floors
├── Existing Windows
├── Daylight
└── Entrance / Environment
↓
Interior Intelligence
├── Room Requirements
├── Room Allocation
├── Floor Plan
├── Room Graph
├── Doors
├── Circulation
├── Furniture
└── Lighting
↓
Minecraft World

---

## 🎯 39. RWM vs. Simple Map Conversion

RWM 不只是：

Map → Blocks

而是：

Real World
↓
Geospatial Data
↓
Geographic Reconstruction
↓
World Understanding
↓
Building Understanding
↓
Interior Understanding
↓
Procedural Reconstruction
↓
Minecraft

這讓 RWM 同時處理：

- Geographic Position
- Real-World Scale
- Terrain
- Environment
- Roads
- Buildings
- Building Types
- Building Geometry
- Existing Windows
- Daylight
- Rooms
- Doors
- Spatial Topology
- Circulation
- Furniture
- Lighting
- 3D Objects
- Outside Terrain

---

## 📦 40. Data Sources & Attribution

RWM 使用公開與第三方資料來源。

主要來源：

- OpenStreetMap

地理、人造環境、道路、建築、水域及其他 OSM Objects。

- Overpass API

用於取得 OpenStreetMap 查詢資料。

RWM 可使用多個公開 Overpass endpoint。

- Nominatim

用於 OpenStreetMap Reverse Geocoding。

- Overture Maps

用於建築及其他地理資料補充。

RWM 使用其公開資料與相關 STAC / S3 資料來源。

- Mapterhorn

Elevation / Terrain Data。

- AWS Terrain Tiles

Elevation Data。

- USGS 3DEP

Regional / High-resolution Elevation Data。

- ESA WorldCover

Land Cover Classification。

- 3DMR

Third-party 3D Models。

- Wikimedia Commons

公開媒體與部分 3D Model 資源。

- Wikidata

3D / Landmark / Metadata information。

- RWM Bundled / Local Assets

# RWM 自身 Bundled Assets、Local Assets、NBTX / Minecraft Assets 與 Local Cache。

RWM 不擁有上述第三方資料。

使用生成結果時，使用者應自行遵守相應資料來源的：

- License
- Attribution Requirements
- Terms of Use
- Data Usage Policies

RWM 的第三方資料來源可能隨服務狀態、資料版本、API 政策與供應商變更而調整。

---

## ⚖️ 41. Independence

RWM 目前作為獨立專案持續開發。

RWM 的設計方向是：

Independent Product
+
Local-First
+
CLI-First
+
Multi-Source
+
Open Data
+
Modular World Engine

- RWM 不以單一官方服務作為核心世界生成能力的依賴。

- RWM 也不依賴 Arnis / ArnisWorld 的官方 API 或官方世界資料服務作為核心資料取得架構。

- RWM 的資料來源與 World Reconstruction Pipeline 由 RWM 自己的架構管理。

---

## 🧪 42. Current Development Status

目前核心方向已形成：

- ✅ Real-World Geographic Reconstruction
- ✅ BBox-based World Generation
- ✅ 1:1 Spatial Mapping
- ✅ Multi-Source Data Processing
- ✅ OSM Integration
- ✅ Overpass Integration
- ✅ Nominatim Integration
- ✅ Overture Integration
- ✅ Multi-Source Elevation
- ✅ Mapterhorn Elevation
- ✅ AWS Terrain Tiles
- ✅ USGS 3DEP
- ✅ Land Cover Integration
- ✅ ESA WorldCover Integration
- ✅ 3D Model Integration
- ✅ Local Asset / NBTX System
- ✅ Bundled Asset Support
- ✅ Terrain Reconstruction
- ✅ Road Reconstruction
- ✅ Building Reconstruction
- ✅ Building Exterior Generation
- ✅ Existing Window Extraction
- ✅ Building Intelligence
- ✅ Building Type Recognition / Classification
- ✅ Room Requirements
- ✅ Room Allocation
- ✅ Procedural Floor Plans
- ✅ Room Graph / Spatial Topology
- ✅ Entrance Analysis
- ✅ Interior Door Generation
- ✅ Interior Circulation
- ✅ Room-Aware Furniture
- ✅ Interior Lighting
- ✅ Daylight Analysis
- ✅ Outside Terrain
- ✅ Outside Padding
- ✅ Java World Output
- ✅ Bedrock World Output
- ✅ Luanti / Minetest Output
- ✅ CLI-First Architecture
- ✅ Local-First Architecture
- ✅ Multi-Source / Resilient Architecture

部分更進階的 Climate / Biome / Environment Intelligence 能力仍可持續擴充。

---

## 🚀 Vision

RWM 最終希望建立的不是：

«「一個把地圖轉成 Minecraft 的工具。」»

而是一個：

«Real-World Reconstruction Engine»

讓 Minecraft 世界從：

Real World Data

開始，

經過：

Data
↓
Understanding
↓
Reconstruction
↓
Procedural Generation

最後成為：

# 🌍 A Minecraft World Built From The Real World

# RWM — Real World Minecraft

«把真實世界帶進 Minecraft，不只是複製它的地圖，而是重建它的空間。»

---

### 📬 聯繫創作者

- Instagram：[a370373/XRH](https://instagram.com/a370373)
- 本人17歲🤔 做的不好請見諒
- 獨立開發 ＆ AI協作
- 緩慢更新 ＆ 除錯
- 持續開發中…👀

---

## 👀作品 & 產品 集

- [RWM- 1:1 Real World Minecraft](https://github.com/a370373/RWM-Real-World-Minecraft)
- 持續增加中…👀

---

### 🤖 AI 協作

RWM 由 a370373/XRH 發起、設計與開發。

開發過程中使用 OpenAI ChatGPT 作為 AI 協作夥伴，協助進行 技術分析、程式碼檢查、除錯 & 文件整理。

產品方向、設計理念 & 最終決策由專案創作者負責。
