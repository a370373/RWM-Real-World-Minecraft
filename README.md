🗺️ RWM — Real World Minecraft

> 讓 Minecraft 世界建立在真實地球的空間尺度上。



RWM（Real World Minecraft）是一個基於 Arnis 世界生成引擎發展而來的獨立 Minecraft 真實世界生成平台。

RWM 讓使用者選擇地球上的任意區域，取得公開的地理、建築、道路、Elevation、Land Cover、Climate 與 3D 資料，經由 RWM World Engine 進行資料處理、融合與世界重建，最終輸出可直接使用的 Minecraft 世界。

RWM 的核心不是：

> 「把地圖放進 Minecraft。」



而是：

> 「讓 Minecraft 世界真正建立在地球上。」 🌍⛏️




---

1. 🌍 核心產品體驗

RWM 的核心流程：

📍 選擇真實世界位置
        ↓
📐 設定 BBox
        ↓
⚙️ 設定世界生成參數
        ↓
🌐 RWM 取得公開資料
        ↓
🧠 RWM World Engine
        ↓
🏗️ 真實世界重建
        ↓
🎮 Minecraft World

使用者可以選擇：

🏙️ 城市

🏘️ 社區

🏫 校園

🏢 建築群

🛣️ 道路

🌳 自然環境

⛰️ 山丘與地形

🌊 海岸與水域

🏛️ 著名地點


最終得到：

> 一個按照真實世界地理位置與實際空間尺度建立的 Minecraft 世界。




---

2. 🧭 RWM 的產品定位

RWM 不定位為：

> 「Arnis 的修改版」



而是：

> 基於 Arnis 世界生成引擎發展而來的獨立 Real World Minecraft 平台。



關係：

Arnis
  │
  │ 世界生成引擎 / 技術基礎
  ▼
RWM
  │
  ├── 自己的產品定位
  ├── 自己的 CLI
  ├── 自己的資料處理
  ├── 自己的資料融合
  ├── 自己的世界生成邏輯
  ├── 自己的 Outside Terrain
  ├── 自己的資產系統
  └── 自己的產品發展方向

因此：

> Arnis = 技術基礎之一
RWM = 建立在其基礎上的獨立產品




---

3. 📍 Geographic Selection

BBox 地理範圍

使用者透過：

min latitude
min longitude
max latitude
max longitude

決定：

> 「我要哪一塊地球？」



例如：

25.03680,121.51010
        ↓
25.03950,121.51300

RWM 將指定的地理區域轉換為 Minecraft 空間。


---

4. 📏 Real-World Scale

RWM 以真實世界地理資料與實際距離作為 Minecraft 空間的基礎。

核心流程：

Real World Coordinates
        ↓
Geographic Projection
        ↓
Minecraft Coordinates

支援：

Scale

Local Projection

Web Mercator Projection



---

5. 🌍 1:1 Real-World Reconstruction

這是 RWM 最核心的產品概念之一。

在預設：

Scale = 1

的情況下：

> 1 公尺真實世界 ≈ 1 個 Minecraft block



例如：

100 公尺道路
      ↓
約 100 blocks

1 公里
      ↓
約 1,000 blocks

10 公里
      ↓
約 10,000 blocks

因此 RWM 不是把真實世界縮小後放進 Minecraft。

而是：

Real-World Coordinates
        ↓
Real-World Distance
        ↓
Geographic Projection
        ↓
Minecraft Coordinates
        ↓
1:1 Spatial Reconstruction

真實世界中的：

道路位置

建築位置

Building Footprint

城市結構

河流

水域

海岸線

地形高度

地理相對位置


都以真實世界資料作為空間基準。

因此：

> RWM 世界可以被理解為「真實世界的 Minecraft 空間重建」。



需要注意：

> 1:1 指的是空間尺度與地理位置的映射，並不代表每一棟建築的每一個 Minecraft block 都能與現實世界完全一致。



實際建築高度、模型細節、植被與物件表現，仍取決於公開資料的解析度、完整程度以及 RWM 的生成能力。


---

6. 🌐 Multi-Source Data Architecture

RWM 不依賴單一地圖或資料來源。

它是一個：

> Multi-Source Real-World Reconstruction System



整體：

OSM
Overture
Elevation
Land Cover
Climate
3D Models
      │
      ▼
RWM Data Processing
      │
      ▼
RWM World Engine

不同資料來源負責不同類型的真實世界資訊。


---

7. 🗺️ OpenStreetMap

OSM 是 RWM 的主要地理與人造環境資料來源之一。

可以提供：

🛣️ 道路

🏠 建築

🌊 水域

🏫 公共設施

🏛️ 地標

🚂 鐵路

🌳 公園

🌉 橋樑

其他 OSM objects



---

8. 🏢 Overture Maps

Overture Maps 用於補充 OSM 可能缺失的建築資料。

主要用途：

OSM Buildings
       +
Overture Buildings
       ↓
More Complete Building Dataset

主要包括：

Building Footprints

建築幾何

OSM 缺失的建築



---

9. ⛰️ Elevation System

RWM 使用真實世界高度資料重建地形。

主要資料來源包括：

Mapterhorn

高解析度 Elevation 來源之一。

用於：

山丘

山脈

高地

低地

山谷

海岸地形


Regional Providers

依照地區使用適合的高解析度資料來源。

AWS Terrain Tiles

Legacy / 備援 Elevation 來源。

約 30m 級別資料。

因此：

High Resolution Providers
        ↓
Regional Providers
        ↓
AWS Terrain Tiles
        ↓
Elevation System
        ↓
Terrain Reconstruction


---

10. 🌱 Land Cover System

RWM 可以使用土地覆蓋資料判斷真實世界地表。

例如 ESA WorldCover：

Forest

Grassland

Cropland

Built-up

Water

Bare / sparse vegetation

Snow / ice

其他土地覆蓋類型


流程：

Land Cover
    ↓
RWM Environment Generator
    ↓
Minecraft Terrain / Blocks / Vegetation


---

11. 🌦️ Climate Data

RWM 也支援氣候分類資料。

例如：

> Köppen–Geiger Climate Data



可作為：

氣候判斷

生態環境判定

自然環境生成

植被決策

未來 Biome 系統


的資料基礎。


---

12. 🏛️ 3D Model System

RWM 不只生成 Minecraft blocks。

它也可以使用第三方 3D 資料。

3DMR

用於取得第三方 3D 模型。

可以應用於：

建築

城市物件

地標

其他真實世界模型


Wikimedia / Wikidata

可提供：

3D Models

Landmark Data

Model Metadata

Licensing Information



---

13. 💾 Local-First Asset System

RWM 的資產系統採用：

> Local-First Asset Resolution



也就是：

Bundled RWM Model
        ↓
Local Cache
        ↓
Network Source

如果模型已經存在於 RWM 本地資產庫：

> 不需要再次從網路下載。



例如 3DMR：

Bundled RWM Model
        ↓
Local Cache
        ↓
3DMR Network Fallback

因此：

> 網路主要用於取得即時或本地不存在的資料。



而模型、資產與 Cache 可以由本地提供。


---

14. 🧱 RWM Bundled Assets

RWM 可以攜帶自己的本地資產。

例如：

🌳 Trees

🚗 Cars

🚤 Boats

🏗️ Cranes

🏠 Props

Minecraft schematic assets

Bundled 3D Models


架構：

Third-Party Models
        +
RWM Bundled Assets
        +
Local Cache
        ↓
3D Object Layer


---

15. 🧠 RWM World Engine

所有資料最終進入：

> RWM World Engine



這是 RWM 的核心。

🌍 REAL WORLD
                         │
       ┌─────────────────┼─────────────────┐
       │                 │                 │
      OSM            Overture          Elevation
       │                 │                 │
       │                 │       ┌─────────┴─────────┐
       │                 │       │                   │
       │                 │   Mapterhorn          Regional/AWS
       │                 │       │                   │
       └─────────────────┼───────┴───────────────────┘
                         │
                  ESA WorldCover
                         │
                    Climate Data
                         │
                  3DMR / Wikimedia
                         │
                         ▼
                🧠 RWM WORLD ENGINE
                         │
          ┌──────────────┼──────────────┐
          │              │              │
          ▼              ▼              ▼
      Structures       Terrain      Environment
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                   3D Object Layer
                         │
                         ▼
                  Minecraft World


---

16. 🏙️ Real-World Structure Generation

Buildings

Building Footprint
        ↓
Building Generator
        ↓
Minecraft Building

Roads

OSM Road
    ↓
Road Generator
    ↓
Minecraft Road

Water

OSM Water
    ↓
Water Generator
    ↓
Minecraft Water

以及：

Bridges

Railways

Parks

Facilities

Street Objects

Other OSM Objects



---

17. ⛰️ Terrain Reconstruction

Real Elevation
      ↓
Elevation Processing
      ↓
Terrain Reconstruction
      ↓
Minecraft Terrain

形成：

山丘

山脈

山谷

高低差

海岸

真實地形起伏


而不是單純的平面 Minecraft 地圖。


---

18. 🌳 Environment Generation

RWM 將：

Land Cover
+
Elevation
+
Climate
+
Water

融合成自然環境。

例如：

Forest
 ↓
Trees + Vegetation

Grassland
 ↓
Grass + Natural Terrain

Water
 ↓
Water + Aquatic Environment

因此生成世界不只是：

> 「建築貼圖」。



而是包含真實世界自然環境。


---

19. 🌊 Outside Terrain System

這是 RWM 的重要產品功能之一。

RWM 不只處理 BBox。

使用者可以決定：

> 「真實世界區域外面是什麼？」



目前包括：

Ocean

Superflat

Normal

Void

Desert

Snow



---

🌊 Ocean

BBox 外可以生成：

海床

Sandstone

Sand

Water

固定海平面


並讓海平面與 RWM BBox 世界高度對齊。


---

🟩 Superflat

BBox 外生成 RWM 控制的超平坦區域。

Minecraft World
       ↓
     Void
       ↓
   RWM BBox
       ↓
Outside Superflat

不是：

> 整張世界變 Superflat。



而是：

> 真實世界 BBox 被放在一個由 RWM 控制的外部環境中。




---

🌍 Normal

Outside Normal 使用 RWM 自己的外部地形生成邏輯，而不是單純依賴 Minecraft 原生世界生成器。

因此更接近：

> RWM-Compatible Outside Terrain




---

20. 📐 Outside Padding

使用者可以控制：

Outside Padding = ?

決定 BBox 周圍額外生成多少空間。

┌─────────────────────────────┐
│                             │
│       Outside Terrain       │
│                             │
│    ┌───────────────────┐    │
│    │                   │    │
│    │   REAL WORLD      │    │
│    │      BBOX         │    │
│    │                   │    │
│    └───────────────────┘    │
│                             │
│       Outside Terrain       │
│                             │
└─────────────────────────────┘

核心規則：

> RWM BBox 永遠是 authoritative area。



Outside Terrain 不應覆蓋 BBox 裡的真實世界資料。


---

21. 🎮 Minecraft World Output

RWM 可以輸出多種世界格式。

☕ Java Edition

Minecraft Java World
        ↓
Anvil
        ↓
region/r.x.z.mca

可以直接放入 Minecraft Java 世界目錄。

📱 Bedrock Edition

輸出：

.mcworld

🌱 Luanti / Minetest

輸出：

map.sqlite


---

22. ⚙️ CLI-First

RWM 採用：

> CLI-First Product Architecture



核心功能不依賴 GUI。

例如：

rwm \
--bbox min_lat,min_lon,max_lat,max_lon \
--output-dir <path> \
--outside-terrain ocean \
--outside-padding 200

CLI 可以控制：

BBox

Output

Scale

Projection

Ground Level

Outside Terrain

Outside Padding

Generation Mode

Overture

3D Models

Interior

Spawn

Rotation

Lighting

Map Preview

Map Item

Game Mode

World Time

Elevation Source


GUI 可以作為：

> RWM 的使用者介面層



而不是 RWM 核心本身。


---

23. 🖥️ User Experience

RWM 最終將複雜的 GIS / Data Processing 工作隱藏在 Engine 內部。

使用者只需要：

📍 Location
      ↓
📐 BBox
      ↓
⚙️ Options
      ↓
🌊 Outside Terrain
      ↓
▶️ Generate
      ↓
🌐 Data Acquisition
      ↓
🧠 RWM World Engine
      ↓
🏗️ World Reconstruction
      ↓
🎮 Minecraft World

使用者不需要理解：

GIS

DEM

STAC

GeoTIFF

Parquet

S3

Row Groups

Projection

Elevation Tiles

Land Cover Dataset


這些全部屬於：

> RWM Internal Engine / Data Processing Layer




---

24. 🧩 Independent & Resilient Architecture

RWM 的核心設計理念不是依賴單一中央服務。

RWM
              │
       ┌──────┼──────┐
       │      │      │
     Local   Cache  Network
       │      │      │
       └──────┼──────┘
              │
        RWM World Engine

RWM 可以：

使用本地 Bundled Assets

使用 Local Cache

取得即時公開資料

使用多個資料來源

使用不同 Elevation Providers

在部分資料來源不可用時使用替代來源


因此：

> RWM 的世界生成能力不應依賴單一官方伺服器。



網路主要負責：

> 取得真實世界資料。



而不是：

> 提供 RWM 本身的核心生成能力。




---

25. 🏗️ 完整產品架構

🗺️ RWM
          REAL WORLD MINECRAFT
                     │
                     ▼
          ┌─────────────────────┐
          │  Geographic Input   │
          │        BBox         │
          └──────────┬──────────┘
                     │
                     ▼
             🌐 DATA SOURCES
                     │
       ┌─────────────┼─────────────┐
       │             │             │
       ▼             ▼             ▼
      OSM         Overture      Elevation
                                   │
                             ┌─────┴─────┐
                             │           │
                        Mapterhorn    Regional/AWS
       │             │             │
       └─────────────┼─────────────┘
                     │
             ┌───────┴────────┐
             │                │
        WorldCover       Climate Data
             │                │
             └───────┬────────┘
                     │
               3D DATA SOURCES
                     │
             ┌───────┴────────┐
             │                │
           3DMR          Wikimedia
             │                │
             └───────┬────────┘
                     │
                     ▼
             💾 LOCAL-FIRST
                     │
          ┌──────────┴──────────┐
          │                     │
     Bundled Assets        Local Cache
          │                     │
          └──────────┬──────────┘
                     │
                     ▼
              🧠 RWM WORLD ENGINE
                     │
       ┌─────────────┼─────────────┐
       │             │             │
       ▼             ▼             ▼
   Structures      Terrain     Environment
       │             │             │
       └─────────────┼─────────────┘
                     ▼
               🧱 3D OBJECTS
                     │
             ┌───────┴────────┐
             │                │
      Third-Party Models   RWM Assets
             │                │
             └───────┬────────┘
                     │
                     ▼
             🌊 OUTSIDE TERRAIN
                     │
          ┌──────────┼──────────┐
          │          │          │
        Ocean    Superflat    Normal
          │          │          │
          └──────────┼──────────┘
                     │
                     ▼
             🎮 WORLD OUTPUT
                     │
       ┌─────────────┼─────────────┐
       │             │             │
      Java        Bedrock       Luanti
       │             │             │
      Anvil       .mcworld     map.sqlite
                     │
                     ▼
                 🌍 RWM WORLD


---

26. 🧩 六大核心能力

因此 RWM 可以正式濃縮成：

① 🌍 Real-World Mapping

將真實世界轉換成 Minecraft。

② 📐 1:1 Spatial Reconstruction

以真實世界空間尺度建立 Minecraft 世界。

> Default Scale = 1 → 約 1m : 1 block



③ 🗺️ Geographic Selection

自由選擇地球上的任意區域。

④ 🏙️ Real-World Structures

重建真實道路、建築、城市與其他地理結構。

⑤ ⛰️ Terrain & Environment Reconstruction

融合 Elevation、Land Cover、Climate、Water 等資料重建自然環境。

⑥ 🎮 Minecraft World Generation

將所有資料融合成真正可使用的 Minecraft 世界。


---

🎯 最終產品定位

> 🗺️ RWM — Real World Minecraft

A Real-World 1:1 Minecraft World Reconstruction Platform.

RWM 是一個基於 Arnis 世界生成引擎發展而來的獨立 Minecraft 世界生成平台。

RWM 整合 OpenStreetMap、Overture Maps、Mapterhorn、AWS Terrain Tiles、ESA WorldCover、氣候資料、3DMR、Wikimedia 等多種公開資料來源，透過 RWM World Engine 將真實世界的地理、建築、道路、地形、土地覆蓋、自然環境與 3D 物件重建為 Minecraft 世界。

在預設 1:1 Scale 下，真實世界的空間距離會以約 1 公尺 : 1 Minecraft block 的尺度映射至 Minecraft。

RWM 採用 Multi-Source、Local-First、CLI-First 的架構。網路主要用於取得即時或本地不存在的真實世界資料，而 RWM 的核心世界生成能力、Bundled Assets 與本地模型資產不依賴單一中央服務。

使用者只需要選擇地球上的位置、設定 BBox 與世界生成選項，RWM 就能完成資料取得、處理、融合與世界生成，最終輸出可直接使用的 Minecraft 世界。

不是「把地圖放進 Minecraft」。

而是「讓 Minecraft 世界真正建立在地球上」。 🌍⛏️

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

## ⚖️ Data & Attribution

RWM itself does not own the underlying geographic data.
The generated world may contain data originating from
third-party datasets.

Users are responsible for complying with the licenses,
terms of use, attribution requirements, and usage policies
of the respective data providers.


#📬 聯繫創作者

Instagram：[a370373/XRH](https://instagram.com/a370373)

#🤖 AI 協作

RWM 由 a370373/XRH 發起、設計與開發。

開發過程中使用 OpenAI ChatGPT 作為 AI 協作夥伴，協助進行 技術分析、程式碼檢查、除錯 & 文件整理。

產品方向、設計理念 & 最終決策由專案創作者負責。
