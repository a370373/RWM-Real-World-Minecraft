##🗺️ RWM — Real World Minecraft

«讓 Minecraft 世界建立在真實地球的空間尺度上。»

RWM（Real World Minecraft）是一個以真實世界地理資料為基礎的 Minecraft 世界生成平台。

使用者可以選擇地球上的任意區域，RWM 取得公開的地理、建築、道路、地形、土地覆蓋、氣候與 3D 資料，經過資料處理、融合與程序化世界生成後，建立一個具有真實地理位置、空間尺度與環境特徵的 Minecraft 世界。

RWM 的目標不是單純：

«「把地圖放進 Minecraft。」»

而是：

«「讓 Minecraft 世界真正建立在地球上。」»

---

## 1. 🌍 Real-World Geographic Selection

RWM 從真實地球上的地理範圍開始。

使用者透過 Bounding Box（BBox） 指定需要生成的區域：

- Min Latitude
- Min Longitude
- Max Latitude
- Max Longitude

BBox 決定：

«RWM 要重建哪一塊真實世界。»

可以是：

- 一棟建築周邊
- 一個街區
- 一個社區
- 一座校園
- 一個城市區域
- 一段道路
- 山區
- 海岸
- 任意指定的地理範圍

BBox 是 RWM 世界生成的核心空間邊界。

---

## 2. 📐 Real-World Spatial Mapping

RWM 將真實世界地理座標轉換為 Minecraft 空間座標：

Real-World Coordinates → Geographic Projection → Minecraft Coordinates

支援：

- Scale
- Local Projection
- Web Mercator Projection
- 真實距離映射

在預設：

Scale = 1

時：

«約 1 公尺真實世界 ≈ 1 個 Minecraft Block»

因此道路、建築、河流、水域、城市結構與地形之間的空間關係，都以真實世界資料作為基礎。

需要注意：

«1:1 代表地理位置與空間尺度的映射，不代表公開資料可以精確還原現實中的每一個 Minecraft Block。»

實際細節取決於資料來源的解析度、完整程度以及 RWM 的生成能力。

---

## 3. 🌐 Multi-Source Real-World Data

RWM 不依賴單一資料來源。

它是一個：

«Multi-Source Real-World Reconstruction System»

不同資料來源負責不同類型的真實世界資訊。

主要資料類型包括：

- OpenStreetMap
- Overture Maps
- Elevation Data
- Land Cover Data
- Climate Data
- 3D Models
- Landmark / Metadata Data

所有資料最終進入 RWM Data Processing Layer，再由 RWM World Engine 進行融合與世界重建。

---

## 4. 🗺️ OpenStreetMap

OpenStreetMap（OSM）提供主要的地理與人造環境資料。

可以提供：

- 道路
- 建築
- 水域
- 鐵路
- 公園
- 橋樑
- 公共設施
- 地標
- 其他 OSM Objects

這些資料構成 RWM 世界中的主要人造環境。

---

## 5. 🏢 Overture Maps

Overture Maps 用於補充 OSM 可能缺失或不足的建築資料。

主要包括：

- Building Footprints
- 建築幾何
- 建築位置
- OSM 缺失的建築

RWM 可以將不同來源的建築資料進行處理與融合，以建立更完整的 Building Dataset。

---

## 6. ⛰️ Elevation System

RWM 使用真實世界 Elevation 資料重建地形。

資料來源可以包括：

- Mapterhorn
- Regional Elevation Providers
- AWS Terrain Tiles
- 其他可用的 Elevation Sources

用於重建：

- 山丘
- 山脈
- 高地
- 低地
- 山谷
- 海岸地形
- 真實世界高低差

資料流程：

Elevation Data → Elevation Processing → Terrain Reconstruction → Minecraft Terrain

---

## 7. 🌱 Land Cover System

土地覆蓋資料可以用於判斷真實世界地表類型。

例如：

- Forest
- Grassland
- Cropland
- Built-up
- Water
- Bare / Sparse Vegetation
- Snow / Ice

土地覆蓋資料可以參與：

Land Cover → Environment Generation → Minecraft Terrain / Vegetation

使自然環境不只是隨機生成，而是具有真實世界資料基礎。

---

## 8. 🌦️ Climate System

RWM 可以使用氣候分類資料，例如：

Köppen–Geiger Climate Data

作為：

- 氣候判斷
- 生態環境判定
- 植被生成
- 自然環境生成
- 未來 Biome 系統

的資料基礎。

---

## 9. 🏛️ 3D Model & Landmark System

RWM 不只生成 Minecraft Blocks，也可以使用第三方 3D 資料。

例如：

- 3DMR
- Wikimedia
- Wikidata

可以提供：

- 建築模型
- 城市物件
- 地標
- 真實世界 3D Models
- Model Metadata
- Licensing Information

這些資料會進入 RWM 的 3D Object Layer。

---

## 10. 💾 Local-First Asset System

RWM 採用：

«Local-First Asset Resolution»

資產解析優先順序：

Bundled RWM Assets → Local Cache → Network Source

如果 RWM 已經擁有某個模型或資產：

«不需要再次從網路取得。»

網路主要用於取得：

- RWM 本地不存在的資料
- 即時公開資料
- 新的第三方資料

因此 RWM 的核心生成能力不需要依賴單一中央服務。

---

## 11. 🧱 RWM Asset System

RWM 可以擁有自己的 Bundled Assets。

例如：

- Trees
- Cars
- Boats
- Cranes
- Props
- Minecraft Schematics
- Bundled 3D Models

資產會與第三方模型及 Local Cache 一起進入：

RWM 3D Object Layer

---

## 12. 🧠 RWM World Engine

所有真實世界資料最終進入：

«RWM World Engine»

這是 RWM 的核心。

它負責：

1. 資料解析
2. 地理座標處理
3. 多來源資料融合
4. 世界空間轉換
5. 結構生成
6. 地形生成
7. 自然環境生成
8. 3D Object Generation
9. 建築室內生成
10. Minecraft World Output

整體流程：

Real World Data → Data Processing → Data Fusion → RWM World Engine → Minecraft World

---

## 13. 🏙️ Real-World Structure Engine

RWM 將真實世界中的人造環境轉換為 Minecraft 結構。

包括：

- Buildings
- Roads
- Bridges
- Railways
- Waterways
- Parks
- Facilities
- Street Objects
- Other Geographic Objects

例如：

Real Road Data → Road Generator → Minecraft Road

Building Footprint → Building Generator → Minecraft Building

---

## 14. 🏠 Building Intelligence System

RWM 的建築生成不只停留在：

Building Footprint → Minecraft Building

未來的 Building Engine 會進一步分析建築：

Building Data → Building Analysis → Building Purpose → Interior Generation

也就是讓 RWM 嘗試理解：

«「這棟建築是做什麼的？」»

建築類型可以根據真實世界 Metadata、建築幾何與其他可用資料進行判斷。

當真實資料不足時，RWM 可以使用程序化規則進行合理推測。

---

## 15. 🧠 Procedural Interior Engine

RWM Interior Engine 的目標不是單純在建築裡放家具。

而是：

«建立合理的建築空間。»

完整流程：

Building Type → Room Requirements → Floor Plan → Rooms → Doors → Connectivity → Furniture → Decoration

因此 RWM 可以從：

«「這是一棟建築」»

進一步生成：

«「這是一棟住宅，所以它應該具有住宅合理的空間結構。」»

---

## 16. 🏢 Building Type & Room System

不同建築具有不同的空間需求。

🏠 Residential

住宅：

- 客廳
- 廚房
- 臥室
- 浴室

🍽️ Restaurant

餐廳：

- 用餐區
- 廚房
- 廁所
- 儲藏室

🏫 School

學校：

- 教室
- 辦公室
- 廁所
- 走廊

🏥 Hospital

醫院：

- 病房
- 診間
- 走廊
- 護理站

🏪 Shop

商店：

- 商品區
- 收銀區
- 倉庫

🏢 Office

辦公樓：

- 辦公區
- 會議室
- 茶水間

未來可以擴充更多建築類型，例如：

- Hotel
- Library
- Factory
- Apartment
- Museum
- Stadium
- Fire Station
- Police Station

---

## 17. 📐 Procedural Floor Plan Generation

RWM 不應使用單一固定室內模板套用所有建築。

Floor Plan Engine 會根據：

- 建築 footprint
- 建築尺寸
- 建築形狀
- 可用樓層空間
- 建築類型
- 房間需求

建立適合該建築的空間配置。

例如住宅：

Small Building → 少量必要房間

Medium Building → 完整住宅空間

Large Building → 更多房間與更複雜的空間配置

因此每棟建築都可以具有不同的 Floor Plan。

---

## 18. 🚪 Door & Entrance System

門不是隨機裝飾，而是建築空間結構的一部分。

RWM 將處理兩種主要入口：

Exterior Entrance

Outside → Building

如果真實世界資料提供：

- Entrance
- Main Entrance
- Door

RWM 優先使用真實資料。

如果沒有，則根據：

- 建築外牆
- 建築入口方向
- 道路關係
- 空間可用性

推測合理入口。

Interior Doors

Room → Room

RWM 尋找房間之間的共享牆，並在合理位置建立門。

門的位置需要考慮：

- 房間連通性
- 共享牆
- 房間空間
- 家具位置
- 建築結構

因此門不是：

«隨機找一個座標放門。»

而是：

«根據 Floor Plan 與房間關係建立合理通行路徑。»

---

## 19. 🕸️ Room Connectivity

RWM 的房間可以形成空間連通關係。

例如：

Entrance → Living Room → Kitchen

Living Room → Bedroom → Bathroom

Floor Plan Generator 會檢查：

«建築內的重要房間是否可以從入口抵達？»

如果房間被完全封閉，系統可以重新調整房間配置或門的位置。

這使建築室內具有真正的：

«Spatial Connectivity»

而不只是視覺上的房間分割。

---

## 20. 🪑 Room-Aware Furniture Generation

家具由房間用途決定。

流程：

Building → Room → Room Type → Furniture Profile → Placement

例如：

Living Room

- Sofa
- Table
- Chairs
- Lighting
- Decorations

Kitchen

- Counter
- Stove
- Sink
- Refrigerator
- Cabinets

Bedroom

- Bed
- Wardrobe
- Bedside Table

Bathroom

- Toilet
- Sink
- Shower

不同房間使用不同的家具規則。

家具位置也會考慮：

- 房間大小
- 牆壁
- 門
- 通道
- 已放置物件

避免家具阻塞主要通行空間。

---

## 21. 🎲 Procedural Variation

RWM 不希望所有同類建築長得完全一樣。

生成結果可以根據：

- 建築尺寸
- 建築形狀
- 樓層
- 房間需求
- Random Seed
- 可用空間

產生不同配置。

因此：

«同樣是住宅，也可以具有不同 Floor Plan。»

同樣的原則也適用於家具與裝飾。

---

## 22. ⛰️ Terrain Reconstruction

真實 Elevation 資料進入 Terrain Engine：

Real Elevation → Processing → Terrain Reconstruction

生成：

- 山丘
- 山脈
- 山谷
- 高低差
- 海岸
- 真實地形起伏

RWM 世界因此不只是平面地圖，而是具有真實世界地形結構的 Minecraft 世界。

---

## 23. 🌳 Environment Generation

RWM 將多種自然環境資料進行融合：

Land Cover + Elevation + Climate + Water

生成：

- Trees
- Vegetation
- Grass
- Natural Terrain
- Water Environment
- 生態環境

未來可以進一步發展：

- Biome Mapping
- Regional Vegetation
- Climate-Based Generation
- Ecosystem Generation

---

## 24. 🌊 Outside Terrain System

RWM BBox 外的世界可以由使用者決定。

可選擇：

- Ocean
- Superflat
- Normal
- Void
- Desert
- Snow

Outside Terrain 是 RWM 真實世界區域之外的環境。

核心規則：

«BBox 是 authoritative area。»

也就是：

BBox 內 → 真實世界重建

BBox 外 → 使用者指定的 Outside Terrain

Outside Terrain 不應覆蓋 BBox 內的真實世界資料。

---

## 25. 📏 Outside Padding

使用者可以指定：

«BBox 周圍額外生成多少區域。»

因此可以建立：

Real-World Area + Controlled Outside Environment

例如：

Real Taiwan Region + Ocean

或：

Real City + Superflat

或：

Real Mountain Area + Void

讓使用者自由決定真實世界區域與 Minecraft 世界其他部分的關係。

---

## 26. 🎮 Minecraft World Output

RWM 最終將生成真正可使用的 Minecraft 世界。

目前支援的輸出方向包括：

Java Edition

Minecraft Anvil World：

"region/r.x.z.mca"

Bedrock Edition

".mcworld"

Luanti / Minetest

"map.sqlite"

World Output Layer 負責將 RWM Engine 的結果寫入對應的世界格式。

---

## 27. ⚙️ CLI-First Architecture

RWM 採用：

«CLI-First Product Architecture»

核心 Engine 不依賴 GUI。

使用者可以透過 CLI 控制：

- BBox
- Output Directory
- Scale
- Projection
- Ground Level
- Outside Terrain
- Outside Padding
- Generation Mode
- Overture
- 3D Models
- Interior
- Spawn
- Rotation
- Lighting
- Map Preview
- Map Item
- Game Mode
- World Time
- Elevation Source

GUI 可以存在，但它只是：

«RWM 的使用者介面層。»

而不是 RWM 的核心。

---

## 28. 🖥️ User Experience

RWM 將複雜的 GIS、資料下載與世界生成工作隱藏在 Engine 內部。

一般使用者只需要：

選擇位置 → 設定 BBox → 選擇生成設定 → Generate → 取得 Minecraft World

使用者不需要直接理解：

- GIS
- DEM
- STAC
- GeoTIFF
- Parquet
- S3
- Projection
- Elevation Tiles
- Land Cover Dataset

這些都屬於：

«RWM Internal Data & Processing Layer»

---

## 29. 🧩 Independent & Resilient Architecture

RWM 的核心不依賴單一中央服務。

架構：

Local Assets + Local Cache + Multiple Public Data Sources + RWM World Engine

RWM 可以：

- 使用 Bundled Assets
- 使用 Local Cache
- 使用公開資料
- 使用多個資料來源
- 使用不同 Elevation Providers
- 在部分來源不可用時使用替代來源

因此：

«RWM 的核心世界生成能力不應依賴單一官方伺服器。»

網路主要負責取得真實世界資料，而不是提供 RWM 本身的核心生成能力。

---

## 30. 🌍 Complete RWM Architecture

                         🌍 REAL WORLD
                              │
                              ▼
                       📍 Geographic BBox
                              │
                              ▼
                    📐 Spatial Mapping
                              │
                              ▼
                    🌐 DATA ACQUISITION
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
         OSM              Overture           Elevation
          │                   │                   │
     Land Cover            Climate          3D Models
          │                   │                   │
          └───────────────────┼───────────────────┘
                              ▼
                    🧠 DATA PROCESSING
                              │
                              ▼
                     🔀 DATA FUSION
                              │
                              ▼
                    🧠 RWM WORLD ENGINE
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
 🏙️ STRUCTURES          ⛰️ TERRAIN           🌳 ENVIRONMENT
        │                     │                     │
        │                     │                     │
        ▼                     ▼                     ▼
 🏢 BUILDING ENGINE      Elevation           Land Cover
        │                                      Climate
        ▼                                      Water
 🧠 BUILDING ANALYSIS
        │
        ▼
 🏠 INTERIOR ENGINE
        │
        ├── Building Type
        ├── Room Planning
        ├── Floor Plan
        ├── Room Segmentation
        ├── Entrance
        ├── Doors
        ├── Connectivity
        ├── Furniture
        └── Decoration
        │
        └─────────────────────┐
                              │
                              ▼
                       🧱 3D OBJECT LAYER
                              │
                  ┌───────────┴───────────┐
                  │                       │
           RWM Bundled Assets       Third-Party Models
                  │                       │
                  └───────────┬───────────┘
                              ▼
                       🌊 OUTSIDE TERRAIN
                              │
                              ▼
                       🎮 WORLD OUTPUT
                              │
                 ┌────────────┼────────────┐
                 │            │            │
               Java        Bedrock      Luanti
                 │            │            │
              Anvil       .mcworld    map.sqlite
                 │            │            │
                 └────────────┼────────────┘
                              ▼
                         🌍 RWM WORLD


---

## 31. 🎯 RWM 的核心價值

RWM 最終希望提供的不只是：

«Real Map → Minecraft»

而是一個完整的：

«Real World → Data → Understanding → Reconstruction → Minecraft»

系統。

它同時處理：

地理位置

真實距離

地形

自然環境

道路與城市結構

建築

建築用途

房間

門與空間連通

家具與室內物件

3D Models

Outside Terrain

最後才輸出：

«一個可以在 Minecraft 中探索的真實世界。»

---

## 🌍 RWM

Real World Data

↓

Geographic Reconstruction

↓

World Understanding

↓

Procedural World Generation

↓

Minecraft

«把真實世界帶進 Minecraft，不只是複製它的地圖，而是重建它的空間。»

---

## 🌍 World Data Sources

RWM generates Minecraft worlds using publicly available
geospatial datasets and services.

### OpenStreetMap (OSM)
Used for:
- Roads
- Buildings
- Water features
- Other mapped geographic objects

### Overture Maps
Used for:
- Building / geographic feature data
- Additional real-world map objects

### Mapterhorn
Used for:
- Elevation / terrain data

### ESA WorldCover 2021
Used for:
- Land-cover classification
- Surface / terrain material inference

RWM does not use a proprietary world database.
Data is fetched from the corresponding public/
third-party services during generation.

---

## ⚖️ Data & Attribution

RWM itself does not own the underlying geographic data.
The generated world may contain data originating from
third-party datasets.

Users are responsible for complying with the licenses,
terms of use, attribution requirements, and usage policies
of the respective data providers.

---

## 📬 聯繫創作者

### Instagram：[a370373/XRH](https://instagram.com/a370373)

---

## 🤖 AI 協作

RWM 由 a370373/XRH 發起、設計與開發。

開發過程中使用 OpenAI ChatGPT 作為 AI 協作夥伴，協助進行 技術分析、程式碼檢查、除錯 & 文件整理。

產品方向、設計理念 & 最終決策由專案創作者負責。
