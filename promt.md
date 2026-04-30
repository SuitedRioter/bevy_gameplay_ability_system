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

3. 增强内置需求库 (可选)

  当前内置需求有设计问题，可以：
  - 重新设计为需要明确的 MaxHealth 属性
  - 添加更多实用的内置需求（距离检查、标签组合等）
  - 为内置需求添加完整测试

  4. 属性捕获快照模式 (高级功能)

  UE GAS 支持两种模式：
  - Snapshot: 在效果创建时捕获属性值
  - Dynamic: 每次重新评估
  - 这对于复杂的 buff/debuff 计算很有用
