我想的Ability模块设计。
1. Ability定义类，为一个struct。放的是能力的配置数据。
2. 当把一个Ability定义类赋予玩家时，就生成一个AbilitySpec的实体，这个实体代表玩家身上的这个技能，内部包含组件如下：AbilitySpec，AbilityActiveState（内部包含是否激活，激活次数信息），AbilityCooldown，AbilityOwner
3. 当一个AbilitySpec的实体激活时，就生成一个AbilitySpecInstance实体（这个我想作为AbilitySpec的子实体实现，这样我在移除AbilitySpec能力时，就可以把他对应的多个AbilitySpecInstance实体，
但是需要在销毁AbilitySpecInstance实体时调用behavior.end方法），他内部要有包含：bIsActive：标记当前实例是否活跃 bIsBlockingOtherAbilities：控制是否阻止其他能力 bIsCancelable：控制当前实例是否可取消这三个属性的组件，用于激活过程中的逻辑控制。另外AbilitySpecInstance实体本质是Ability定义类的复制。用于在激活过程中调用behavior的逻辑。
4. 方法的参数尽量不要使用world，这个会降低性能，除非只能通过world来实现。


我需要优化现有abilities模块的system和监听system的设计。
1. on_try_activate_ability 这个监听入口需要改一下，生成的AbilitySpec Entity需要是激活者的child entity，这样激活者Entity被移除时，能自动移除子实体。
2.


这个（/Users/zhengwei/GeneralProject/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities）文件夹下是UnrealEngine的GAS插件的代码，我希望你在当前项目使用bevy来实现UnrealEngine的GAS插件的功能，
对外功能表现必须与原模块一致，你需要把原模块oop思想的代码已bevy的ecs思想实现，现有项目已经实现了一部分代码。你可以参考并优化（当需要优化的时候）。我希望你参考已经完成的中文设计文档（在./docs/design_document_cn.md），然后再进行复刻,注意，我已经自己实现了GameplayTag的功能（源码在/Users/zhengwei/RustProject/bevy_gameplay_tag），相关需要使用GameplayTag，GameplayTagContainer，GameplayTagCountContainer的直接使用。如果需要查看bevy的api，可以分析bevy源码（源码在/Users/zhengwei/RustProject/bevy）


这个（/Users/zhengwei/GeneralProject/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities）文件夹下是UnrealEngine的GAS插件的代码，我希望你在当前项目使用bevy来实现UnrealEngine的GAS插件的功能，
对外功能表现必须与原模块一致，你需要把原模块oop思想的代码已bevy的ecs思想实现，现有项目已经实现了一部分代码。你可以参考并优化（当需要优化的时候）。注意，我已经自己实现了GameplayTag的功能（源码在/Users/zhengwei/RustProject/bevy_gameplay_tag），相关需要使用GameplayTag，GameplayTagContainer，GameplayTagCountContainer的直接使用。如果需要查看bevy的api，可以分析bevy源码（源码在/Users/zhengwei/RustProject/bevy），我不需要实现联网功能，目标是做单机游戏。
在写代码之前：
1. 先解释架构映射关系
2. 识别等价实现
3. 标出潜在风险点
【转换铁律】
1. 不再有 Actor、Component、UObject。一切都是实体(Entity)加纯数据组件(Component)。
2. 任何有状态的变量都必须成为组件，不可内聚在系统里。
3. 所有函数实现（包括虚函数）都变为系统，通过 Query 组合获取数据依赖。
4. 继承树被扁平化，差异用标记组件或枚举字段代替。


这个（/Users/zhengwei/GeneralProject/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities）文件夹下是UnrealEngine的GAS插件的代码，我希望你在当前项目使用Godot-CSharp来实现UnrealEngine的GAS插件的功能，也就是我希望从UE复刻到适配Godot的GameplayAbilities，
对外功能表现必须与原模块一致，现有项目下已经复刻了一部分，在GameplayAbilities文件夹，你可以在此基础上继续复刻，补全或者优化现有代码。注意，我已经自己实现了GameplayTag的功能（就在当前项目的GameplayTag目录下），相关需要使用GameplayTag，GameplayTagContainer，GameplayTagCountContainer的直接使用。我不需要实现联网功能，目标是做单机游戏。
在写代码之前：
1. 先解释架构映射关系
2. 识别等价实现
3. 标出潜在风险点
