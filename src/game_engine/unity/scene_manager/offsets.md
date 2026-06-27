# todo

- do other unity versions NOT have the extra dereference for mono object?
- check the asr can load the classes (log 'em)
- check the other scenes loaded
    - can we get "is loading"?

# project setup

- scene 1
    - toplevelgame
        - children:
            - childgame1
            - childgame2 (inactive)
        - classes:
            - RestartScript
    - other default GOs under
        - disable MainCamera audio listener
- scene 2
    - (empty)

build profiles:

- w/ dev on/off
- scene list has both

player settings:

- ScriptingBackend Mono/Il2Cpp
- Active Input Handling - Old

# lib

## restart script

```cs
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using UnityEngine;
using UnityEngine.SceneManagement;

public class RestartScript : MonoBehaviour
{
    static int myStatic = 0x73399007;
    public int special = 0x191B1D1F;
    public int FPS = 60;

    // Start is called before the first frame update
    void Start()
    {
        QualitySettings.vSyncCount = 1;

        Application.targetFrameRate = FPS;
    }

    // Update is called once per frame
    void Update()
    {
        Time.timeScale = 1f;

        if (Input.GetKeyDown(KeyCode.R))
        {
            UnityEngine.Debug.Log("reloading scene");
            SceneManager.LoadSceneAsync(SceneManager.GetActiveScene().name);
        }

        if (Input.GetKeyDown(KeyCode.D))
        {
            UnityEngine.Debug.Log("loading scene 1 additively");
            SceneManager.LoadSceneAsync("scene 1", LoadSceneMode.Additive);
        }

        if (Input.GetKeyDown(KeyCode.F))
        {
            UnityEngine.Debug.Log("loading scene 2 additively");
            SceneManager.LoadSceneAsync("scene 2", LoadSceneMode.Additive);
        }

        if (Input.GetKeyDown(KeyCode.C))
        {
            UnityEngine.Debug.Log("loading scene 1 singly");
            SceneManager.LoadSceneAsync("scene 1");
        }

        if (Input.GetKeyDown(KeyCode.V))
        {
            UnityEngine.Debug.Log("loading scene 2 singly");
            SceneManager.LoadSceneAsync("scene 2");
        }
    }
}
```

## lua sig script

```lua
-- Cheat Engine Lua
-- Resolves the FINAL absolute address from an instruction matched by AOB.
--
-- Example:
--   sig      = "48 8B 05 ?? ?? ?? ?? 48 85 C0"
--   dispOff  = 3   -- offset inside instruction where relative displacement starts
--   instrLen = 7   -- total instruction length
--
-- This reproduces what CE shows in disassembler for RIP-relative instructions.

function sig(sig, dispOffset)
    local scan = AOBScan(sig)
    if not scan or scan.Count == 0 then
        print("sig not found")
        return nil
    end

    local instr = tonumber(scan[0], 16)
    scan.destroy()

    -- read signed rel32
    local rel = readInteger(instr + dispOffset)

    if rel >= 0x80000000 then
        rel = rel - 0x100000000
    end

    -- rel32 is ALWAYS relative to NEXT instruction
    local final = instr + dispOffset + 4 + rel

    return final
end
```

# Unity 2023.1.22f1

## 64 bit, Windows, Mono

```
is_dev_build = false
go_dev = 0x10 -- size of EditorExtensions
co_dev = 0x8

pointer_size = 0x8

scene_manager = sig("48 83 EC 20 4C 8B ?5 ?? ?? ?? ?? 33 F6", 7, 7)

loaded_scenes = 0x8
scene_count = 0x18
active_scene = 0x48
dont_destroy_on_load_scene = 0x70

asset_path = 0x10
build_index = 0x98
root_storage_container = 0xF0

prev = 0x0
next = 0x8
current = 0x10

game_object = 0x30 + (is_dev_build and go_dev or 0)
game_object_name = 0x60 + (is_dev_build and go_dev or 0)
active_self = 0x56 + (is_dev_build and go_dev or 0)
active_in_hierarchy = 0x57 + (is_dev_build and go_dev or 0)
children = 0x70 + (is_dev_build and go_dev or 0)

classes = game_object
class = 0x28 + (is_dev_build and co_dev or 0)
class_name = 0x48
```

## 64 bit mac

```lua
is_dev_build = false
go_dev = 0x10 -- size of EditorExtensions
co_dev = 0x8

pointer_size = 0x8

scene_manager = sig("41 54 53 50 4C 8B ?5 ???????? 41 83", 7, 7)

loaded_scenes = 0x8
scene_count = 0x18
active_scene = 0x48
dont_destroy_on_load_scene = 0x70

asset_path = 0x10
build_index = 0x98
root_storage_container = 0xF0

prev_node = 0x0
next_node = 0x8
current_node = 0x10

game_object = 0x30 + (is_dev_build and go_dev or 0)
game_object_name = 0x60 + (is_dev_build and go_dev or 0)
active_self = 0x5E + (is_dev_build and go_dev or 0)
active_in_hierarchy = 0x5F + (is_dev_build and go_dev or 0)
children = 0x70 + (is_dev_build and go_dev or 0)

classes = game_object
class_mono = 0x18 + (is_dev_build and co_dev or 0)
-- has extra dereference
-- component.m_MonoReference->raw->vtable->class->name
class_name_mono = 0x40
class_il2cpp = 0x18
class_name_il2cpp = 0x10
```

# Unity 6000.4.5f1

## 64 bit windows

```lua
is_dev_build = false
go_dev = 0x10 -- size of EditorExtensions
co_dev = 0x8

pointer_size = 0x8

scene_manager = sig("48 83 EC 20 48 8B 2D ?? ?? ?? ?? 33 F6", 7, 7)

loaded_scenes = 0x8
scene_count = 0x18
active_scene = 0x48
dont_destroy_on_load_scene = 0x70

asset_path = 0x10
build_index = 0x98
root_storage_container = 0xF0

prev = 0x0
next = 0x8
current = 0x10

game_object = 0x20 + (is_dev_build and go_dev or 0)
game_object_name = 0x50 + (is_dev_build and go_dev or 0)
active_self = 0x46 + (is_dev_build and go_dev or 0)
active_in_hierarchy = 0x47 + (is_dev_build and go_dev or 0)
children = 0x48 + (is_dev_build and go_dev or 0)

classes = game_object
class_mono = 0x28 + (is_dev_build and co_dev or 0)
class_name_mono = 0x48
class_il2cpp = 0x18
class_name_il2cpp = 0x10
```

## 64 bit mac

Checked for mono and il2cpp.

```lua
is_dev_build = false
go_dev = 0x10 -- size of EditorExtensions
co_dev = 0x8

pointer_size = 0x8

scene_manager = sig("41 54 53 50 4C 8B ?5 ???????? 41 83", 7, 7)

loaded_scenes = 0x8
scene_count = 0x18
active_scene = 0x48
dont_destroy_on_load_scene = 0x70

asset_path = 0x10
build_index = 0x98
root_storage_container = 0xF0

prev_node = 0x0
next_node = 0x8
current_node = 0x10

game_object = 0x20 + (is_dev_build and go_dev or 0)
game_object_name = 0x50 + (is_dev_build and go_dev or 0)
active_self = 0x46 + (is_dev_build and go_dev or 0)
active_in_hierarchy = 0x47 + (is_dev_build and go_dev or 0)
children = 0x48 + (is_dev_build and go_dev or 0)

classes = game_object
class_mono = 0x18 + (is_dev_build and co_dev or 0)
-- has extra dereference
-- component.m_MonoReference->raw->vtable->class->name
class_name_mono = 0x40
class_il2cpp = 0x18
-- also has extra dereference
-- component.m_MonoReference->raw->vtable->class->name
class_name_il2cpp = 0x10
```